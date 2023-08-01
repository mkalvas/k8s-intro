# Practical Kubernetes Intro

A short introduction to Kubernetes through practical examples.

## Prerequisites

1. Docker
2. Minikube — `brew install minikube`

## Project structure

- `hello-api` — the simplest, stateless API service to begin deploying into k8s
- `db-api` — a simple API that connects to a `mysql` database
- `mysql` — a single instance mysql deployment with a persistent volume and secrets, meant for backing the `db-api`

## The Course

_Note_: For the remainder of the course we'll assume that you have a minikube cluster running locally (`minikube start`) and your `kubectl` is configured to use that cluster. We'll also assume all the commands listed are run from the project root unless specifically stated otherwise.

We'll start by creating a docker image for our [`hello-api`](./hello-api). The [`Dockerfile`](./hello-api/Dockerfile) already exists, we simply need to create our docker image in a way that minikube will be able to find. This step is basically like publishing the image to a registry like ECR.

```sh
# tell minikube that we're using our local docker environment
eval $(minikube docker-env)

# build a versioned tag of our simple api
docker build -t hello-api:v1 ./hello-api
```

Next, we need to get this image to run in kubernetes. This brings us to the topic of [deployments](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/). A deployment is a declarative definition of the desired cluster state for [Pods](https://kubernetes.io/docs/concepts/workloads/pods/) and [ReplicaSets](https://kubernetes.io/docs/concepts/workloads/controllers/replicaset/). In order to run a container in a Kubernetes pod, we need to create a deployment.

The [`hello-api/k8s/deployment.yaml`](./hello-api/k8s/deployment.yaml) file contains the definition for our API. We'll apply this to our cluster using `kubectl`.

```sh
kubectl apply -f ./hello-api/k8s/deployment.yaml
```

To see whether we set everything up right let's get info about the deployment.

```sh
kubectl get deployment hello-api-deployment -o wide
# Your AGE field will be different
# NAME                   READY   UP-TO-DATE   AVAILABLE   AGE     CONTAINERS   IMAGES         SELECTOR
# hello-api-deployment   1/1     1            1           3m13s   hello-api    hello-api:v1   app=hello-api

kubectl get pods -o wide
# Your AGE field and your pod's suffix IDs will be different 
# NAME                                   READY   STATUS    RESTARTS   AGE     IP           NODE
# hello-api-deployment-984499c6f-85m44   1/1     Running   0          5m38s   10.244.0.4   minikube
```

Great! Our API is running but let's pause to see what's going on. If we inspect our deployment a little more closely using the `kubectl describe` command, we can see how the declarative specification works.

```sh
kubectl describe deploy hello-api-deployment
# Name:                   hello-api-deployment
# Namespace:              default
# Labels:                 app=hello-api
# Annotations:            deployment.kubernetes.io/revision: 1
# Selector:               app=hello-api
# Replicas:               1 desired | 1 updated | 1 total | 1 available | 0 unavailable
# StrategyType:           RollingUpdate
# MinReadySeconds:        0
# RollingUpdateStrategy:  25% max unavailable, 25% max surge
# Pod Template:
#   Labels:  app=hello-api
#   Containers:
#    hello-api:
#     Image:      hello-api:v1
#     Port:       <none>
#     Host Port:  <none>
#     Limits:
#       cpu:        500m
#       memory:     128Mi
#     Environment:  <none>
#     Mounts:       <none>
#   Volumes:        <none>
# Conditions:
#   Type           Status  Reason
#   ----           ------  ------
#   Available      True    MinimumReplicasAvailable
#   Progressing    True    NewReplicaSetAvailable
# OldReplicaSets:  <none>
# NewReplicaSet:   hello-api-deployment-984499c6f (1/1 replicas created)
# Events:
#   Type    Reason             Age   From                   Message
#   ----    ------             ----  ----                   -------
#   Normal  ScalingReplicaSet  8s    deployment-controller  Scaled up replica set hello-api-deployment-984499c6f to 1
```

In the description, you can see a lot information about our deployment including the pod template we're using to create containers, the `ReplicaSets`, and the events that have been seen and handled by the cluster.

Let's make a change. In the [`./hello-api/k8s/deployment.yaml`](./hello-api/k8s/deployment.yaml) file, change the `replicas` field to `2`. Now, apply our changed spec and see what happens.

```sh
kubectl apply -f ./hello-api/k8s/deployment.yaml
kubectl get pods
# NAME                                   READY   STATUS 
# hello-api-deployment-984499c6f-bgqz5   1/1     Running
# hello-api-deployment-984499c6f-bl8lg   1/1     Running

kubectl describe deployment hello-api
# NewReplicaSet:   hello-api-deployment-984499c6f (2/2 replicas created)
# Events:
#   Type    Reason             Age   From                   Message
#   ----    ------             ----  ----                   -------
#   Normal  ScalingReplicaSet  11m   deployment-controller  Scaled up replica set hello-api-deployment-984499c6f to 1
#   Normal  ScalingReplicaSet  61s   deployment-controller  Scaled up replica set hello-api-deployment-984499c6f to 2 from 1
```

Near the bottom we can see that the cluster saw a difference between the specification of our deployment (that we just applied) and the status of the (old) deployment in the cluster. It then took the appropriate action to create another `ReplicaSet` for our deployment and we now have two pods running. This is the real magic of Kubernetes. You can declare what you'd like the state of your cluster to be and it will handle everything else.

Now we're going to finish setting up the API by allowing it to receive traffic. Right now, there's nothing exposing the container in the pod to the cluster or the outside world. In order to do that, we need to create a [Service](https://kubernetes.io/docs/concepts/services-networking/service/).

We'll apply the [`hello-api/k8s/service.yaml`](./hello-api/k8s/service.yaml) file and then make a curl request to our app.

```sh
kubectl apply -f ./hello-api/k8s/service.yaml

kubectl get service -o wide
# NAME         TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)          AGE   SELECTOR
# hello-api    NodePort    10.96.228.70   <none>        3000:32315/TCP   14m   app=hello-api
```

_Note_: You'll want to know and understand [the different possible `TYPE`s of services](https://kubernetes.io/docs/concepts/services-networking/service/#publishing-services-service-types).

### Minikube on Mac with Docker Engine

Since we're running minikube on a mac with the docker engine, we'll need to run a minikube command to expose the cluster to our browser. This step isn't needed in other environments and the service created would have an `EXTERNAL-IP` that we could use to access our service.

```sh
# Do this in a second terminal and leave it running
minikube service hello-api

# Your output will have different urls
# |-----------|-----------|-------------|---------------------------|
# | NAMESPACE |   NAME    | TARGET PORT |            URL            |
# |-----------|-----------|-------------|---------------------------|
# | default   | hello-api |        3000 | http://192.168.67.2:32315 |
# |-----------|-----------|-------------|---------------------------|
# 🏃  Starting tunnel for service hello-api.
# |-----------|-----------|-------------|------------------------|
# | NAMESPACE |   NAME    | TARGET PORT |          URL           |
# |-----------|-----------|-------------|------------------------|
# | default   | hello-api |             | http://127.0.0.1:62426 |
# |-----------|-----------|-------------|------------------------|
# 🎉  Opening service default/hello-api in default browser...
```

## State, Persistence, and Volumes

To show an example of how to attach persistent state to a cluster, we're going to spin up a new API called `db-api` and a `mysql` database. For now we'll use a single `mysql` instance and gloss over [StatefulSets](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/). If you need to run multiple instances of a database with one primary and many read only replicas, you'll want to use `StatefulSets`.

We can deploy an instance of `mysql` if we follow the steps above and use the official `mysql` image instead of our API's but it won't be connected to any storage and therefore lose all of our data whenever the pod goes up or down. This is obviously not what we're looking for in a database like this, so we need to create a [`PersistentVolume`](https://kubernetes.io/docs/concepts/storage/persistent-volumes/) to persist the data in the cluster and a [`PersistentVolumeClaim`](https://kubernetes.io/docs/concepts/storage/persistent-volumes/#persistentvolumeclaims) for our `mysql` instance to be able to lay claim to using that volume.

We'll also create a [`Secret`](https://kubernetes.io/docs/concepts/configuration/secret/) for storing the password.

_Note_: Kubernetes secrets are fairly insecure by default (see the linked page), so we'll need to use something more robust like an external secrets manager or encrypted secrets in a production environment. This example is for learning only.

```sh
kubectl apply -f ./mysql/secret.yaml
kubectl apply -f ./mysql/volume.yaml
kubectl apply -f ./mysql/volume-claim.yaml
kubectl apply -f ./mysql/deployment.yaml
kubectl apply -f ./mysql/service.yaml
```

You'll need to wait for a minute if you're downloading and creating the `mysql` container for the first time. You can watch its progres with `kubectl get pods -w`. If all is well, you'll see the container status transition to `Running` and will be able to shell into the running instance with the following command.

```sh
kubectl exec $(kubectl get pod -l app=mysql -o jsonpath='{.items[0].metadata.name}') -it -- /bin/bash
```

You can then use `mysql -p` and enter the password in the [`./mysql/secret.yaml`](./mysql/secret.yaml) file to log in. Let's create an example user table for our `db-api` to read. I've written that service to expect only an integer `id` field and a string `name` field in the `test` database.

```sh
create database test;

use test;

create table `test`.`users` (
  `id` int not null auto_increment,
  `name` varchar(255) null,
  primary key (`id`));

insert into users (name) values ('mike');

select * from users;
# +----+------+
# | id | name |
# +----+------+
# |  1 | mike |
# +----+------+
```

Following the steps from above for the API portion, we build the image and deploy the pods and the service for `db-api`.

```sh
# if this is a different shell than you've been using, you might need to run
#   eval $(minikube docker-env)
# again for docker to put this in the right place and have minikube find it
docker build -t db-api:v1 ./db-api
kubectl apply -f ./db-api/k8s/deployment.yaml
kubectl apply -f ./db-api/k8s/service.yaml
```

Again (for macs with docker engine) we can connect to our service and see that it's running and connected to mysql!

```sh
# in a second tab (your url will be different)
minikube service db-api
curl "http://127.0.0.1:63872/users"
# [{"id":1,"name":"mike"}]
```

So how does this work exactly? Our service definitions are the key. Let's take a quick look at the [`db-api/k8s/deployment.yaml`](./db-api/k8s/deployment.yaml)

```yaml
# ... omitted
containers:
  - name: db-api
    image: db-api:v1
    env:
      - name: DB_HOST
        value: mysql # this is the name of the Service we create for mysql
      - name: DB_PASS
        valueFrom:
          secretKeyRef:
            name: mysql-secret
            key: password
# ... omitted
```

Our service uses an environment variable called `DB_HOST`. We read this value and use it in the connection string for our ORM.

```rs
// From our db-api source code
let host = std::env::var("DB_HOST").unwrap_or("localhost".to_string());
let password = std::env::var("DB_PASS").unwrap_or("testpassword".to_string());
format!("mysql://root:{password}@{host}:3306/test")
```

Notice that I've included some defaults here, but our app will get these values from the cluster: `DB_HOST` will be `mysql` and `DB_PASS` will be the value we defined in the [`./mysql/secret.yaml`](./mysql/secret.yaml) in the `password` key. The most important piece here is the fact that the cluster manages hostname discovery based on these service names. Let's look at the services again.

```sh
kubectl get services
# NAME         TYPE        CLUSTER-IP     EXTERNAL-IP   PORT(S)          AGE
# db-api       NodePort    10.108.46.55   <none>        3000:31548/TCP   7m11s
# hello-api    NodePort    10.96.228.70   <none>        3000:32315/TCP   4h24m
# mysql        ClusterIP   10.101.76.93   <none>        3306/TCP         33m
```

The `NAME` field can be used as the hostname for network traffic that's intended for that service. Things using the db can use `mysql` and if something wanted to call one of our APIs it would use either `hello-api` or `db-api`.

## Helm

Now that you know how to deploy things and network them in and out of the cluster, you might be thinking that some of this is a bit tedious. For example, adding all the files, applying of them to the cluster and making sure all these names are synced up in the different files can be error prone and time consuming. This is the motivation for [`helm`](https://helm.sh/). The value proposition for `helm` is to be a package manager for kubernetes.

As the author of the `db-api` app, I know that it needs environment variables `DB_USER` and `DB_PASS`. I also know that it needs to persist its data to a `mysql` database. If you want to run the API, you need to know how to configure it and run it in your cluster. But what if I could package that up and give you something that you could "just install" and it would run in your cluster? Helm accomplishes this via [`Charts`](https://helm.sh/docs/topics/charts/). If I write a chart, then I can publish that [somewhere](https://artifacthub.io/) and you can "just install" `db-api` and its backing `mysql` from that. It also offers some de-duplication, testing, linting, and templating that makes managing all these different cluster resources easy.

I'm not going to get into the creation of a helm chart here, but it'd be a great next step for anyone reading this to test your understanding and take it to the next level.
