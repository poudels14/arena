k8s_yaml(kustomize('./kube/tilt'))

k8s_resource('tilt-arena-postgres', port_forwards=['6001:5432'])
k8s_resource('tilt-arena-arenasql-cluster', port_forwards=['6000:6000'])
# k8s_resource('tilt-arena-minio', port_forwards=['8001:8001', '10001:10001'])

# k8s_resource('tilt-arena-workspace-cluster', port_forwards=['9001:9000'])
# k8s_resource('tilt-arena-app-cluster', port_forwards=['9002:9000'])

docker_build('arenasql-cluster',
  context="./bin",
  dockerfile='./kube/prod/arenasql-cluster/Dockerfile',
)

# docker_build('app-cluster',
#   context="./crates/target/release",
#   dockerfile='./kube/prod/app-cluster/Dockerfile',
# )

# docker_build('workspace-cluster',
#   context="./crates/target/release",
#   dockerfile='./kube/prod/workspace-cluster/Dockerfile',
# )