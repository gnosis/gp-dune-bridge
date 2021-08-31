# Data collector and api for gnosis protocol v2

## Instructions for running the api

Running the api with the data form user_data.json:
```
cargo run
```


and then visit the webpage:

```
http://127.0.0.1:8080/api/v1/profile/0xa4a6ef5c494091f6aeca7fa28a04a219dd0f31b5
```

Running via docker:

1. Fetching data
```
docker build -t fetch_script -f ./docker/Dockerfile.binary .
docker run -e DUNE_PASSWORD='pwd' -e DUNE_USER='user' -ti fetch_script 


docker build -t gpdata -f docker/Dockerfile.binary . 
docker run -ti -e DUNE_DATA_FILE='/usr/local/data/gpdata/user_data.json' gpdata gpdata
or
docker run -ti -e DUNE_DATA_FILE='/usr/local/data/gpdata/user_data.json' -p 8080:8080 gpdata gpdata             
```

## Instructions for getting data from dune


### installation
Cd into dune_data_api
```
cd dune_data_api
```

```
pip install -r requirements.txt
```

### Download data:

Pulling new query results:

```
python store_dune_data.py
```

Update query:
```
python modify_dune_query.py
```