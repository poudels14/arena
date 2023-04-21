k8s_yaml(kustomize('./kube/tilt'))

k8s_resource('tilt-arena-postgres', port_forwards=['6000:5432'])
