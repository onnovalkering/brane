use anyhow::Result;
use brane_clb::grpc::{CallbackKind, CallbackRequest, CallbackServiceClient};
use std::sync::atomic::{AtomicI32, Ordering};
use tonic::transport::Channel;

pub struct Callback {
    application_id: String,
    location_id: String,
    job_id: String,
    event_counter: AtomicI32,
    client: CallbackServiceClient<Channel>,
}

impl Callback {
    ///
    ///
    ///
    pub async fn new<S: Into<String>>(
        application_id: S,
        location_id: S,
        job_id: S,
        callback_to: S,
    ) -> Result<Self> {
        let client = CallbackServiceClient::connect(callback_to.into()).await?;

        Ok(Callback {
            application_id: application_id.into(),
            location_id: location_id.into(),
            job_id: job_id.into(),
            event_counter: AtomicI32::new(1),
            client,
        })
    }

    ///
    ///
    ///
    async fn call<K: Into<i32>>(
        &mut self,
        kind: K,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        let order = self.event_counter.fetch_add(1, Ordering::Release);

        let request = CallbackRequest {
            application: self.application_id.clone(),
            location: self.location_id.clone(),
            job: self.job_id.clone(),
            kind: kind.into(),
            order,
            payload: payload.unwrap_or_default(),
        };

        self.client
            .callback(request)
            .await
            .map(|_| ())
            .map_err(|error| anyhow!(error))
    }

    ///
    ///
    ///
    pub async fn ready(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Ready, payload).await
    }

    ///
    ///
    ///
    pub async fn initialized(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Initialized, payload).await
    }

    ///
    ///
    ///
    pub async fn started(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Started, payload).await
    }

    ///
    ///
    ///
    pub async fn heartbeat(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Heartbeat, payload).await
    }

    ///
    ///
    ///
    pub async fn finished(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Finished, payload).await
    }

    ///
    ///
    ///
    pub async fn stopped(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Stopped, payload).await
    }

    ///
    ///
    ///
    pub async fn failed(
        &mut self,
        payload: Option<Vec<u8>>,
    ) -> Result<()> {
        self.call(CallbackKind::Failed, payload).await
    }
}
