# Text Vectorizer

Text Vectorization API using [all-MiniLM-L12-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L12-v2)


## Requirements

### Rust

The recommended way to install and maintain rust toolchain is via [rustup](https://rustup.rs/)

```sh
rustup toolchain install stable
```

### Dependencies 

Install [libtorch](https://pytorch.org/get-started/locally/) version 2.1

```sh
cd /opt
wget https://download.pytorch.org/libtorch/cu118/libtorch-cxx11-abi-shared-with-deps-2.1.0%2Bcu118.zip
unzip libtorch-cxx11-abi-shared-with-deps-2.1.0+cu118.zip
export LIBTORCH=/opt/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
```

## Build 

```sh
cargo build
```


## Run

```sh
cargo run
```

## Usage

```sh
curl -v \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"text": "Lorem ipsum"}' \
    -H "Authorization: Bearer 123" \
    localhost:5002/predict
```

or

```sh
curl -v \
    -X POST \
    -d 'Lorem ipsum' \
    -H "Authorization: Bearer 123" \
    localhost:5002/predict
```
