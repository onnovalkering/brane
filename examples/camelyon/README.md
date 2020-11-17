# Camelyon demo
In order to run the demo, follow the following steps.

## 1. Preperation
Clone the repository, and [install the Brane CLI](https://onnovalkering.github.io/brane/installation).

## 2. Build the Camelyon package
From the root of the Brane repository:
```shell
$ brane build --context ./examples/camelyon container.yml
```

Verify that the package has been built correctly:
```shell
$ brane list
```

## 3. Start a local Brane deployment
Navigate to the Docker deployment folder, and run `docker-compose`.
```shell
$ cd ./deployment/docker
$ docker-compose up -d
```

## 4. Push Camelyon package to the Brane deployment
From the root of the Brane repository:
```shell
$ brane login http://localhost:8080 -u uc1demo
$ brane push camelyon 1.0.0
```

## 5. Set the deployment-wide Camelyon configuration
From the root of the Brane repository (update the contents accordingly):
```shell
$ ./examples/camelyon/configure.sh
```

## 6. Start a new Jupyter notebook
You're now ready to start the notebook. 

From your client machine, start a SSH tunnel for the `8888` port, if applicable 
```shell
$ export REMOTE_SERVER_HOSTNAME="..."
$ export REMOTE_SERVER_USER="..."

$ ssh -L 8888:127.0.0.1:8888 $REMOTE_SERVER_HOSTNAME -l $REMOTE_SERVER_USER
```

Now execute the DSL code:

__Cell 1__
```go
import "fs"
import "camelyon"
```

__Cell 2__
```go
patient := new Input {
    file_name: "patient_051_node_2",
    imagenet_weights: "resnet101_weights_tf.h5",
    model_weights: "tumor_classifier.h5",
    interpret: true,
    n_samples: 5
}

model := new Model {
    type: "resnet101",
    loss: "binary_crossentrophy",
    activation: "sigmoid",
    lr: "1e-4",
    decay: "1e-6",
    momentum: 0.9,
    nesterov: true,
    batch_size: 32,
    epochs: 15,
    verbose: 1
}

output_dir := new_directory
settings := new Settings {
    training_centres: "0,1,2,3",
    slide_level: 7,
    patch_size: 224,
    n_samples: 500,
    output_dir: output_dir
}
```

__Cell 3__
```go
result := heatmap_of patient with model and settings
```

__Cell 4__
```go
//!display result.heatmap
```

__Cell 5__
```go
//!display result.interpretability[0]
```
