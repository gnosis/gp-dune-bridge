import json
from duneanalytics import DuneAnalytics
from datetime import datetime
import os

# initialize client
dune = DuneAnalytics(os.environ['DUNE_USER'], os.environ['DUNE_PASSWORD'])

# try to login
dune.login()

# fetch token
dune.fetch_auth_token()

# fetch query result id using query id
# query id for any query can be found from the url of the query:
result_id = dune.query_result_id(query_id=135804)

# fetch query result
data = dune.query_result(result_id)

user_data = data["data"]["get_result_by_result_id"]
now = datetime.now()

date_of_data_creation = user_data[0]["data"]["day"][0:10]

data_set = {"user_data": user_data,
            "time_of_download": now.strftime("%d/%m/%Y %H:%M:%S")}

if bool(data_set):
    with open('data/user_data/user_data_from' + date_of_data_creation + '.json', 'w', encoding='utf-8') as f:
        json.dump(data_set, f, ensure_ascii=False, indent=4)
else:
    print("query is still calculating")
