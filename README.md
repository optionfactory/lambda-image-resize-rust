This binary responds to Amazon S3 events and triggers a resize on the uploaded image with the sizes specified.

## Configure

This binary relies on the following env vars:

* `RESIZE_SIZES`, a sequence of comma-separated FOLDER:WIDTHxHEIGHT targets, e.g. (`SIZES=@1x:200x300,@2x:300x400`)
* `RESIZE_DEST_REGION`, name of target bucket's region (e.g 'eu-south-1')
* `RESIZE_DEST_BUCKET`, name of target bucket (e.g. 'mybucket')

## Compile

```
make build
```

You might need to get an up-to-date [Lambda-Rust docker image](https://hub.docker.com/r/softprops/lambda-rust/) to compile this binary.

If that is the case, try: 

```
make deps
```


You can find the (zipped) bootstrap ready for upload to AWS Lambda in `target/lambda/release/bootstrap.zip`
