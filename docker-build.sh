docker build -t jaci/jms:$(cat VERSION) -t jaci/jms:latest -f Dockerfile .
docker build -t jaci/jms-ui:$(cat VERSION) -t jaci/jms-ui:latest -f Dockerfile.ui .

docker-compose build