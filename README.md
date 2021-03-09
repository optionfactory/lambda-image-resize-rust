# Lambda Image Resize

A simple image resize lambda function, written in Rust.

This binary responds to Amazon S3 events and triggers a resize on the uploaded image with the sizes specified. Right now you can only resize on the width of an image.

## Configure

This binary relies on one env var:

* `SIZES`, a sequence of comma-separated FOLDER:WIDTHxHEIGHT targets, e.g. (`SIZES=@1x:200x300,@2x:300x400`)

## Compile

Use [Lambda-Rust docker image](https://hub.docker.com/r/softprops/lambda-rust/) to compile this binary. With Docker running run the following command to build a release.

```
make build
```

You can find the (zipped) bootstrap ready for upload to AWS Lambda in `target/lambda/release/bootstrap.zip`
