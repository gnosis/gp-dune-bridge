import json
from duneanalytics import DuneAnalytics
from datetime import datetime
from pathlib import Path
import os

# Entire history does not need to be downloaded again. do not run query, if the download has been done in the past and file exists
entire_history_path = Path(os.environ['DUNE_DATA_FOLDER'] +
                           "/user_data/user_data_entire_history.json")
if entire_history_path.is_file():
    exit()

# initialize client
dune = DuneAnalytics(os.environ['DUNE_USER'], os.environ['DUNE_PASSWORD'])

# try to login
dune.login()

# fetch token
dune.fetch_auth_token()

# fetch query result id using query id
# query id for any query can be found from the url of the query:
result_id = dune.query_result_id(query_id=157348)

# fetch query result
data = dune.query_result(result_id)

user_data = data["data"]["get_result_by_result_id"]
now = datetime.now()
data_set = {"user_data": user_data,
            "time_of_download": now.strftime("%d/%m/%Y %H:%M:%S")}
if bool(data_set):
    with open(entire_history_path, 'w', encoding='utf-8') as f:
        json.dump(data_set, f, ensure_ascii=False, indent=4)
else:
    print("query is still calculated")
