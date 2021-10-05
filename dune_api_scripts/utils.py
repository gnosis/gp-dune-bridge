
from datetime import datetime
from pathlib import Path
import os
import json


def parse_data_from_dune_query(data):
    user_data = data["data"]["get_result_by_result_id"]
    date_of_data_creation = datetime.strptime(
        user_data[0]["data"]["day"][0:10], '%Y-%m-%d')
    return {"user_data": user_data,
            "time_of_download": date_of_data_creation.strftime("%d/%m/%Y %H:%M:%S")}


def store_as_json_file(data_set):
    file_path = Path(os.environ['DUNE_DATA_FOLDER'] +
                     "/user_data/")
    os.makedirs(file_path,  exist_ok=True)
    with open(os.path.join(file_path, Path("user_data_from" + datetime.now().strftime("%m-%d-%Y") + ".json")), 'w+', encoding='utf-8') as f:
        json.dump(data_set, f, ensure_ascii=False, indent=4)
    print("Written updates into: " + os.path.join(file_path,
          Path("user_data_from" + datetime.now().strftime("%m-%d-%Y") + ".json")))
