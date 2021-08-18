import json
from duneanalytics import DuneAnalytics
from datetime import datetime


# initialize client
dune = DuneAnalytics('user', 'password')

# try to login
dune.login()

# fetch token
dune.fetch_auth_token()

# fetch query result id using query id
# query id for any query can be found from the url of the query:
result_id = dune.query_result_id(query_id=101571)

# fetch query result
data = dune.query_result(result_id)

user_data = data["data"]["get_result_by_result_id"]
now = datetime.now()
data_set = {"user_data": user_data,
            "time_of_download": now.strftime("%d/%m/%Y %H:%M:%S")}

with open('user_data.json', 'w', encoding='utf-8') as f:
    json.dump(data_set, f, ensure_ascii=False, indent=4)
