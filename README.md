# GKE Cluster Notifications

This is a lightweight web service written in Rust using the Axum web framework. It receives [GKE Cluster Notifications](https://cloud.google.com/kubernetes-engine/docs/concepts/cluster-notifications) in the form of Pub/Sub events. Events are formatted, logged, and optionally posted to Slack.

## Installation

1.  Configure environment

    ```
    GCP_PROJECT="my-project"
    ```

2.  Build and push the image

    ```
    docker build --platform linux/amd64 -t "us.gcr.io/${GCP_PROJECT}/gke-cluster-notifications" ./
    docker push "us.gcr.io/${GCP_PROJECT}/gke-cluster-notifications"
    ```

    OR

    ```
    gcloud builds submit \
      --project "${GCP_PROJECT}" --region "us-central1" \
      --tag "us.gcr.io/${GCP_PROJECT}/gke-cluster-notifications"
    ```

3.  Deploy the service

    ```
    gcloud run deploy gke-cluster-notifications \
      --project "${GCP_PROJECT}" --region "us-central1" \
      --ingress=internal --allow-unauthenticated \
      --image "us.gcr.io/${GCP_PROJECT}/gke-cluster-notifications" \
      --set-env-vars "JSON_LOG=true,GCP_PROJECT=${GCP_PROJECT}"
    ```

## Usage

Once the image is built and deployed to Cloud Run, you'll need to [enable cluster notifications](https://cloud.google.com/kubernetes-engine/docs/how-to/cluster-notifications) and configure a Pub/Sub push subscription to receive and send messages to the service on Cloud Run.

When posting to Slack is desired, you will need to [create a Slack App, then enable and create an Incoming Webhook](https://api.slack.com/messaging/webhooks) for the channel where messages will be posted.

### Environment Variables

This service utilizes various environment variables for it's configuration. At a minimum, both `JSON_LOG=true` and `GCP_PROJECT=my-project` should be configured when deploying the service to Cloud Run.

* `JSON_LOG` - Should be either `true` or `false` (the default). When `true`, this enables Stackdriver compatible JSON formatted log output.

* `RUST_LOG` - Configures log levels via `tracing_subscriber::EnvFilter`. For example, a value of `gke_cluster_notifications=debug` will enable debug logging (without enabling debug logging in dependencies) while a value of `debug` will enable debug logs for any crate (including the service itself). By default, a log level of `info` is used.

* `SLACK_WEBHOOK` - Configures an incoming Webhook URL where Slack messages will be sent via JSON POST.

* `GCP_PROJECT` - Pub/Sub messages for cluster notifications do not include the project name. Because of this, the GCP project identifier must be configured via environment variable to avoid the nondescript project number being used in paths, Cloud Console URLs, etc.

## Testing

Running tests:

```
cargo test
```

Internal results of each test can be seen by disabling output capturing. For example:

```
cargo test -- --nocapture log_entry
```

Slack messages can be posted to Slack by setting the `SLACK_WEBHOOK` environment variable and running the relevant test:

```
export SLACK_WEBHOOK=https://hooks.slack.com/services/my/weboook/url
cargo test message::slack::tests::post
```

Slack message blocks can also be previewed by pasting each line of output from `message::slack::tests::post` into the [Block Kit Builder](https://app.slack.com/block-kit-builder/):

```
cargo test -- --nocapture message::slack::tests::post
```
