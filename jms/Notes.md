A local docker container for postgres must be run: 
```bash
docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 127.0.0.1:5432:5432 -d postgres
```