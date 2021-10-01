import json
from duneanalytics import DuneAnalytics
from pathlib import Path
import datetime
import os

# Entire history does not need to be downloaded again. do not run query, if the download has been done in the past and file exists
entire_history_path = Path(os.environ['DUNE_DATA_FOLDER'] +
                           "/user_data")
os.makedirs(entire_history_path, exist_ok=True)
file_entire_history = Path(os.path.join(
    entire_history_path, Path("user_data_entire_history.json")))
if file_entire_history.is_file():
    print("file already downloaded")
    exit()

endDate = "'2022-01-01'"

today = datetime.date.today()
tomorrow = today + datetime.timedelta(days=-1)
startDate = "'2021-03-01'"
endDate = "'" + tomorrow.strftime("%Y-%m-%d") + "'"

# initialize client
dune = DuneAnalytics(os.environ['DUNE_USER'], os.environ['DUNE_PASSWORD'])

# try to login
dune.login()

# fetch token
dune.fetch_auth_token()

file_path = Path(os.environ['APP_DATA_REFERRAL_RELATION_FILE'])
app_data_referral_link = json.loads(
    '{"0x0000000000000000000000000000000000000000000000000000000000000abc": "0x0000000000000000000000000000000000000000"}')
if file_path.is_file():
    with open(file_path) as json_file:
        app_data_referral_link = json.load(json_file)


string_of_pair_app_data_referral = ""

for hash in app_data_referral_link:
    string_of_pair_app_data_referral += "('" + hash+"','" + \
        app_data_referral_link[hash].replace("0", "/", 1)+"'),"

string_of_pair_app_data_referral = string_of_pair_app_data_referral[:-1]

queryStart = """ WITH
-- first two tables are inputs from outside of dune
"""

queryUsers = """
-- This table allows to add users, in case there are users that have not yet traded,
-- though still want to know their potential available trading volume
users as ((
SELECT *
   FROM (
         VALUES (decode('00017f958d2ee523a2206206994597c13d831ec7', 'hex')),
                (decode('36416D81e590FF67370E4523B9Cd3257Aa0A853c', 'hex')),
                (decode('b0e83c2d71a991017e0116d58c5765abc57384af', 'hex')),
                (decode('c143bc1af9f3843db49670e43a2f055a47385d6b', 'hex')),
                (decode('a6ddbd0de6b310819b49f680f65871bee85f517e', 'hex'))
 ) AS t (owner))
 UNION
 (Select
 owner FROM gnosis_protocol_v2."GPv2Settlement_evt_Trade" trades
 WHERE evt_block_time between """ + startDate + " and " + endDate + """
 group by owner )
),
"""

queryAffiliate = """
-- This table provides the mapping between affiliate and appData bidirectional
mapping_appdata_affiliate as (
    SELECT * FROM (VALUES """ + string_of_pair_app_data_referral + """    )as t("appData", affiliate)
    ),
    """
queryConstant = """
-- All tables below this line just help the caluclation of the end result.
trades_with_sell_price AS (
    SELECT
        evt_tx_hash as tx_hash,
        ("sellAmount" - "feeAmount") as token_sold,
        "evt_block_time" as batch_time,
        evt_tx_hash,
        owner,
        "orderUid",
        "sellToken" as sell_token,
        "buyToken" as buy_token,
        ("sellAmount" - "feeAmount")/ pow(10,p.decimals) as units_sold,
        "buyAmount",
        "sellAmount",
        "feeAmount" / pow(10,p.decimals) as fee,
        price as sell_price
    FROM gnosis_protocol_v2."GPv2Settlement_evt_Trade" trades
    LEFT OUTER JOIN prices.usd as p
        ON trades."sellToken" = p.contract_address
        AND p.minute between """ + startDate + " and " + endDate + """
        AND date_trunc('minute', p.minute) = date_trunc('minute', evt_block_time)
   Where evt_block_time between """ + startDate + " and " + endDate + """
   and owner IN (Select * from users)
),

trades_with_prices AS (
    SELECT
        date_trunc('month', batch_time) as month,
        batch_time,
        evt_tx_hash,
        evt_tx_hash as tx_hash,
        owner,
        token_sold,
        "orderUid",
        sell_token,
        buy_token,
        units_sold,
        "buyAmount" / pow(10,p.decimals) as units_bought,
        fee,
        sell_price,
        price as buy_price,
        (CASE
            WHEN sell_price IS NOT NULL THEN sell_price * units_sold
            WHEN sell_price IS NULL AND price IS NOT NULL THEN price * "buyAmount" / pow(10,p.decimals)
            ELSE  -0.01
        END) as trade_value,
        sell_price * fee as fee_value
    FROM trades_with_sell_price t
    LEFT OUTER JOIN prices.usd as p
        ON p.contract_address = (
                CASE
                    WHEN t.buy_token = '\xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee' THEN '\xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2'
                    ELSE t.buy_token
                END)
        AND  p.minute between """ + startDate + " and " + endDate + """
        AND date_trunc('minute', p.minute) = date_trunc('minute', batch_time)
    Where owner IN (Select * from users)
),

-- Provides users stats within GP
user_stats_of_gp as (
SELECT
    date_trunc('month', batch_time) as month,
    count(*) as number_of_trades,
    sum(trade_value) as cowswap_usd_volume,
    sum(fee_value) as cowswap_fee_volume,
    owner
FROM trades_with_prices
GROUP BY 1, owner
ORDER BY owner DESC),

trade_call_data_and_hash as (
SELECT
      jsonb_array_elements(trades) as trade_call_data,
      "call_tx_hash"
    FROM gnosis_protocol_v2."GPv2Settlement_call_settle" call
    where call_block_time between """ + startDate + " and " + endDate + """
    -- and call."call_tx_hash" = '\x88db1ba26eba480f88049ac09fdde46327a9c6fecd07a26dfc2b81cd53bfcb20'
    -- and call."trades"->0->"sellAmount" = evt."sellAmount"
  ),

decoded_trade_call_data_and_hash as (
SELECT
      trade_call_data->'appData' as appdata,
      (trade_call_data->'sellAmount')::numeric as "sellAmount",
      (trade_call_data->'buyAmount')::numeric as "buyAmount",
      (trade_call_data->'feeAmount')::numeric as "feeAmount",
      "call_tx_hash"
    FROM trade_call_data_and_hash
 ),

app_data_and_volumes as(
Select trade_data.month, Replace(app_data.appdata::text, '"', '') as "appData", sum(trade_data."trade_value") as "sum_usd_volume" from decoded_trade_call_data_and_hash app_data
inner join trades_with_prices trade_data
on app_data."call_tx_hash" = trade_data."tx_hash"
and app_data."sellAmount"::numeric = trade_data."token_sold"::numeric
-- and app_data."buyAmount"::numeric = trade_data."buyAmount"::numeric. /// We could do more checks in the future, tbd.
-- //Also it's not yet clear to me whether the current logic of comparison will hold for all time, ie.. if we are starting to charge fees in the buyToken
group by app_data.appdata, trade_data.month),

app_data_and_nr_of_referrals as(
Select  trade_data.month, Replace(app_data.appdata::text, '"', '') as "appData", count(distinct(trade_data.owner)) as "nr_of_referrals" from decoded_trade_call_data_and_hash app_data
inner join trades_with_prices trade_data
on app_data."call_tx_hash" = trade_data."tx_hash"
and app_data."sellAmount"::numeric = trade_data."token_sold"::numeric
-- and app_data."buyAmount"::numeric = trade_data."buyAmount"::numeric. /// We could do more checks in the future, tbd.
-- //Also it's not yet clear to me whether the current logic of comparison will hold for all time, ie.. if we are starting to charge fees in the buyToken
group by app_data.appdata,  trade_data.month),

affiliate_program_results as (
Select
    CASE WHEN app_data_and_nr_of_referrals.month is Null THEN app_data_and_volumes.month ELSE app_data_and_nr_of_referrals.month END as month,
    affiliate as owner,
    sum(sum_usd_volume) as "total_referred_volume",
    sum(nr_of_referrals) as "nr_of_referrals" from mapping_appdata_affiliate
inner join app_data_and_volumes on app_data_and_volumes."appData"::text = mapping_appdata_affiliate."appData"::text
inner join app_data_and_nr_of_referrals on app_data_and_nr_of_referrals."appData"::text = mapping_appdata_affiliate."appData"::text
group by 1, affiliate
),


-- Provides users stats outside of GP
user_stats_general as (
Select
    date_trunc('month', block_time) as month,
    trader_a as owner,
    sum(usd_amount) as usd_volume_all_exchanges
from dex.trades
where block_time between """ + startDate + " and " + endDate + """
and trader_a IN (Select * from users)
group by 1, 2
),

-- First major result table of volumes within GP and outside of GP
trading_volume_results as (
    Select
        Replace( Case When user_stats_general.owner is Null Then user_stats_of_gp.owner::TEXT ELSE user_stats_general.owner::TEXT END, '\', '0') as owner,
        Case When user_stats_general.month is NUll Then user_stats_of_gp.month ELSE user_stats_general.month END as month,
        number_of_trades,
        cowswap_usd_volume,
        usd_volume_all_exchanges
    from user_stats_of_gp
    full outer join user_stats_general
    on (user_stats_general.owner = user_stats_of_gp.owner and user_stats_general.month = user_stats_of_gp.month)
)

-- Select * from trading_volume_results
Select
    CASE WHEN ar.owner is NUll THEN tr.owner ELSE ar.owner END as owner,
    CASE WHEN ar.month is NUll THEN tr.month ELSE ar.month END as day, -- just renaming the month to day to make it compatible to other query 
    total_referred_volume,
    nr_of_referrals,
    number_of_trades,
    cowswap_usd_volume,
    usd_volume_all_exchanges
from affiliate_program_results ar
full outer join trading_volume_results tr
on ar.owner = tr.owner
and tr.month = ar.month
"""

query = queryStart + queryUsers + queryAffiliate + queryConstant
dune.initiate_new_query(query_id=157348, query=query)
dune.execute_query(query_id=157348)
