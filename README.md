# hc-nb-api

This is the service for collecting all NB in an account and storing them in a Postgres DB in Linode. There is a client service here for consuming this data:
[Health Check Client](https://github.com/nathanle/hc-nb-api-client).

This is intended to be a managed component of the service and only requires one copy to be running.

1. Create Postgres database in Linode

2. Create LKE cluster

3. Generate PAT 

4. Update `hc-secrets.yaml` with PAT, DB Password, DB host and port


```yaml
apiVersion: v1
kind: Secret
metadata:
  name: hc-secrets
  namespace: health-check
stringData:
  APIVERSION: v4
  TOKEN: LINODE-PAT 
  MAINDB_PASSWORD: <DB PASSWORD>
  MAINDB_HOSTPORT: 1.2.3.4:5678
```


5. Apply `hc-secrets.yaml`

6. Adjust `hc-deployment.yaml` if needed

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hc-nb-main 
  name: health-check
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hc-nb-main 
  template:
    metadata:
      labels:
        app: hc-nb-main 
    spec:
      serviceAccountName: node-health-check-operator-rs-account
      imagePullSecrets:
      - name: ghcr-login-secret
      containers:
      - name: hc-nb-main 
          image: nathanles/hc-nb-main:latest
        envFrom:
        - secretRef:
            name: hc-secrets
        resources:
          requests:
            memory: "10Mi"
            cpu: "10m"
          limits:
            memory: "6Gi"
            cpu: "1000m"
        imagePullPolicy: IfNotPresent 
```
```
```

7. Apply `hc-deployment.yaml`



![Demo Health Check API](https://github.com/nathanle/nathanle.github.io/blob/main/hc.gif)
