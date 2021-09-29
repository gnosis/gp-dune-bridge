

import json
from duneanalytics import DuneAnalytics
from datetime import datetime
from pathlib import Path
import os

entire_history_path = Path(os.environ['DUNE_DATA_FOLDER'] +
                           "/app_data/")
os.makedirs(entire_history_path, exist_ok=True)
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

with open(os.path.join(entire_history_path, Path("distinct_app_data.json")), 'w+', encoding='utf-8') as f:
    json.dump(data_set, f, ensure_ascii=False, indent=4)
