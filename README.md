# google-cloud-rust
Rust packages for [Google Cloud Platform](https://cloud.google.com/) services.  
Providing a high level API for gRPC API like [Google Cloud Go](https://github.com/googleapis/google-cloud-go).

![CI](https://github.com/yoshidan/google-cloud-rust/workflows/CI/badge.svg?branch=main)

## Announcement
### 2025.02.06:  
Since the development of [google cloud rust by Google](https://github.com/googleapis/google-cloud-rust) seems to have resumed,
We have decided to donate the `google-cloud-*` namespace to Google.
#### Migration from `google-cloud-*` to `gcloud-*`

Library users do not need to modify existing code.
Use `package` option at [dependency] in Cargo.toml
```
google-cloud-spanner = { package="gcloud-spanner", version="1.0.0" }
```

## Component 

* [google-cloud-spanner](./spanner)
* [google-cloud-pubsub](./pubsub)
* [google-cloud-storage](./storage)
* [google-cloud-bigquery](./bigquery)
* [google-cloud-artifact-registry](./artifact-registry)
* [google-cloud-kms](./kms)

## Example
* [google-cloud-rust-example](https://github.com/yoshidan/google-cloud-rust-example)

## License
This project is licensed under the [MIT license](./LICENCE).

## Contributing
Contributions are welcome.
1. Fork this repository.
2. Make changes, commit to your fork.
3. Send a pull request with your changes.
4. Confirm the success of CI.
