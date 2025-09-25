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

6. Apply `hc-deployment.yaml`



![Demo Health Check API](https://github.com/nathanle/nathanle.github.io/blob/main/hc.gif)
