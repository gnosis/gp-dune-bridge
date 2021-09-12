

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
result_id = dune.query_result_id(query_id=142824)

# fetch query result
data = dune.query_result(result_id)

app_data = data["data"]["get_result_by_result_id"]
now = datetime.now()
data_set = {"app_data": app_data,
            "time_of_download": now.strftime("%d/%m/%Y %H:%M:%S")}

with open('distinct_app_data.json', 'w', encoding='utf-8') as f:
    json.dump(data_set, f, ensure_ascii=False, indent=4)
