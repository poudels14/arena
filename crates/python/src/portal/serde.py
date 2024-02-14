import pandas as pd
import json


# Open AI sometimes calls display function
# so, polyfill that function to return value as is
def display(value):
    return value


def serialize(value):
    if isinstance(value, pd.DataFrame):
        return ["dataframe", value.to_json(orient="split")]
    if value is None:
        return None
    return ["json", json.dumps(value)]
