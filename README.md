# GKE Cluster Notifications

This is a lightweight web service written in Rust using the Axum web framework. It receives [GKE Cluster Notifications](https://cloud.google.com/kubernetes-engine/docs/concepts/cluster-notifications) in the form of Pub/Sub events. Events are logged, formatted, and optionally posted to Slack.

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
      --image "us.gcr.io/${GCP_PROJECT}/gke-cluster-notifications" \
      --no-allow-unauthenticated
    ```

### Requirements

- Rust 1.66

## Usage

TODO

## Testing

TODO

## Roadmap

TODO
