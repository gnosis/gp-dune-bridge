import json
from duneanalytics import DuneAnalytics
import os


# initialize client
dune = DuneAnalytics(os.environ['DUNE_USER'], os.environ['DUNE_PASSWORD'])

# try to login
dune.login()

# fetch token
dune.fetch_auth_token()

# execute query again
dune.execute_query(query_id=142824)
