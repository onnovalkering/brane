use crate::{docker, packages, grpc};
use anyhow::{Result, Context as _};
use brane_dsl::{Compiler, CompilerOptions};
use brane_bvm::{VM, VmResult, VmCall};
use brane_bvm::values::Value;
use rdkafka::producer::FutureProducer;
use tonic::{Request, Response, Status};
use std::collections::HashMap;
use specifications::common::Value as SpecValue;
use specifications::package::PackageInfo;
use uuid::Uuid;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct DriverHandler {
    pub producer: FutureProducer,
}

#[tonic::async_trait]
impl grpc::DriverService for DriverHandler {
    ///
    ///
    ///
    async fn create_session(
        &self,
        _request: Request<grpc::CreateSessionRequest>,
    ) -> Result<Response<grpc::CreateSessionReply>, Status> {
        let uuid = Uuid::new_v4().to_string();
        let reply = grpc::CreateSessionReply { uuid };
        Ok(Response::new(reply))
    }

    ///
    ///
    ///
    async fn execute(
        &self,
        request: Request<grpc::ExecuteRequest>,
    ) -> Result<Response<grpc::ExecuteReply>, Status> {
        let request = request.into_inner();

        let options = CompilerOptions::new();
        let package_index = packages::get_package_index().await.unwrap();
        let mut compiler = Compiler::new(options, package_index.clone());

        let function = compiler.compile(request.input)
            .map_err(|_| Status::invalid_argument("Compilation error."))?;

        let mut vm = VM::new(package_index);
        vm.call(function, 1);

        loop {
            match vm.run(None) {
                VmResult::Call(call) => {
                    vm.result(make_function_call(call).await.unwrap());
                },
                VmResult::Ok(value) => {
                    let output = format!("{:?}", value);
                    return Ok(Response::new(grpc::ExecuteReply { output }));
                },
                VmResult::RuntimeError => {
                    return Err(Status::invalid_argument("Runtime error."))
                }
            }
        }
    }
}

///
///
///
async fn make_function_call(
    call: VmCall,
) -> Result<Value> {
    let package_dir = packages::get_package_dir(&call.package, Some("latest"))?;
    let package_file = package_dir.join("package.yml");
    let package_info = PackageInfo::from_path(package_file)?;
    let functions = package_info.functions.expect("Package has no functions!");

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));

    let mut arguments: HashMap<String, SpecValue> = HashMap::new();
    let function = functions.get(&call.function).expect("Function does not exist!");
    for (i, p) in function.parameters.iter().enumerate() {
        arguments.insert(p.name.clone(), call.arguments.get(i).unwrap().as_spec_value());
    }

    let command = vec![
        String::from("--application-id"),
        String::from("test"),
        String::from("--location-id"),
        String::from("localhost"),
        String::from("--job-id"),
        String::from("1"),
        String::from("code"),
        call.function.clone(),
        base64::encode(serde_json::to_string(&arguments)?),
    ];

    let exec = docker::ExecuteInfo::new(image, image_file, None, Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    debug!("stderr: {}", stderr);
    debug!("stdout: {}", stdout);

    let output = stdout.lines().last().unwrap_or_default().to_string();
    let output: Result<SpecValue> = decode_b64(output);
    match output {
        Ok(value) => Ok(Value::from(value)),
        Err(err) => {
            println!("{:?}", err);
            Ok(Value::Unit)
        }
    }
}

///
///
///
fn decode_b64<T>(input: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let input =
        base64::decode(input).with_context(|| "Decoding failed, encoded input doesn't seem to be Base64 encoded.")?;

    let input = String::from_utf8(input[..].to_vec())
        .with_context(|| "Conversion failed, decoded input doesn't seem to be UTF-8 encoded.")?;

    serde_json::from_str(&input)
        .with_context(|| "Deserialization failed, decoded input doesn't seem to be as expected.")
}
