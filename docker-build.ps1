docker build -t jaci/jms:$(cat VERSION) -f Dockerfile .
docker tag jaci/jms:$(cat VERSION) jaci/jms:latest

docker build -t jaci/jms-ui:$(cat VERSION) -f Dockerfile.ui .
docker tag jaci/jms-ui:$(cat VERSION) jaci/jms-ui:latest
