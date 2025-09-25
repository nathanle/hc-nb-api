# hc-nb-api

This is the service for collecting all NB in an account and storing them in a Postgres DB in Linode. There is a client service here for consuming this data:
[Health Check Client](https://github.com/nathanle/hc-nb-api-client).



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




![Demo Health Check API](https://github.com/nathanle/nathanle.github.io/blob/main/hc.gif)
