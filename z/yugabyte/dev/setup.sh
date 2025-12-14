curl -o yugabyte.yaml \
https://raw.githubusercontent.com/yugabyte/yugabyte-db/master/cloud/kubernetes/yugabyte-statefulset-rf-1.yaml


kubectl port-forward -n default svc/yb-tservers 5433:5433

kubectl exec -n default -it yb-tserver-0 -- sh -c "cd /home/yugabyte && ysqlsh -h yb-tserver-0.yb-tservers.default"