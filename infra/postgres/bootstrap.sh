sudo -u postgres psql -f bootstrap.sql

kubectl cp bootstrap.sql yb-tserver-0:/tmp/bootstrap.sql

kubectl exec -n default -it yb-tserver-0 -- sh -c "ysqlsh -h yb-tserver-0.yb-tservers.default -f /tmp/bootstrap.sql"

