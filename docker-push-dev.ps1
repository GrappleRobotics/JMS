docker tag jaci/jms:$(cat VERSION) registry.gitlab.rise.jaci.au/jaci/jms/jms:$(cat VERSION)
docker tag jaci/jms:latest registry.gitlab.rise.jaci.au/jaci/jms/jms:latest
docker tag jaci/jms-ui:$(cat VERSION) registry.gitlab.rise.jaci.au/jaci/jms/jms-ui:$(cat VERSION)
docker tag jaci/jms-ui:latest registry.gitlab.rise.jaci.au/jaci/jms/jms-ui:latest

docker push registry.gitlab.rise.jaci.au/jaci/jms/jms:$(cat VERSION)
docker push registry.gitlab.rise.jaci.au/jaci/jms/jms:latest
docker push registry.gitlab.rise.jaci.au/jaci/jms/jms-ui:$(cat VERSION)
docker push registry.gitlab.rise.jaci.au/jaci/jms/jms-ui:latest