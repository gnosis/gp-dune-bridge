# Data collector and api for gnosis protocol v2

## Instructions for running the api

Running the api with the data form user_data.json:
```
cargo run
```


and then visit the webpage:

```
http://127.0.0.1:8080/api/v1/profile/0xa4a6ef5c494091f6aeca7fa28a04a219dd0f31b5
or
http://127.0.0.1:8080/api/v1/profile/0xe7207afc5cd57625b88e2ddbc4fe9de794a76b0f
```

Running via docker:

1. Fetching data
```
docker build -t fetch_script -f ./docker/Dockerfile.binary .
docker run -e DUNE_PASSWORD=<pwd> -e DUNE_USER=alex@gnosis.pm -e APP_DATA_REFERRAL_RELATION_FILE=/usr/src/app/data/app_data_referral_relationship.json -v /Users/alexherrmann/gnosis/gp-dune-bridge/data/:/usr/src/app/data -ti fetch_script /bin/sh

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
python3 -m venv env
source ./env/bin/activate
pip install -r requirements.txt
```

### Download data:

Pulling new query results:

```
python store_trading_data_for_entire_history.py
python store_trading_data_for_today.py
```


Update query:
```
python modify_and_execute_dune_query_for_overalls_trading_volume.py
python modify_and_execute_dune_query_for_todays_trading_volume.py
```