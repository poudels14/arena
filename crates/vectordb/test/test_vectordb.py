import sys
import os
from os import path
# import numpy as np

if not path.exists("./vectordb.so"):
    raise "vectordb.so missing; creat a symlink from $ROOT/crate/target/debug to 'vectordb.so'"

import vectordb

db = vectordb.Database("./testdb")

# collections = db.list_collections()
# print(collections)

try:
    db.create_collection("collection-1", {"dimension": 4})
except Exception as e:
    print("Error:", e)

print(db.list_collections())

collection_id = "collection-1"
document_id = "document-1"

try:
    db.add_document(
        collection_id,
        document_id,
        {
            "content": b"This is a really awesome document",
        },
    )
    db.add_document(
        collection_id,
        "document-2",
        {
            "content": b"SECOND DOCUMENT",
        },
    )
    db.add_document(
        collection_id,
        "document-3",
        {
            "content": b"THIRD DOCUMENT",
        },
    )
except Exception as e:
    print("Error:", e)


print("---------------------------------------------------")
print("COLLECTIONS =", db.list_collections())
print("---------------------------------------------------")
# print(db.list_documents(collection_id))

document = db.get_document(collection_id, document_id)
print(str(bytes(document["content"]), "UTF-8"))
# # print(str(db.get_document_content(collection_id, document_id), "UTF-8"))


db.set_document_embeddings(
    collection_id,
    # document_id,
    "document-3",
    [
        {"start": 0, "end": 100, "vectors": [0.1, 0.2, 0.3, 0.4]},
        {"start": 100, "end": 200, "vectors": [0.5, 0.5, 0.2, 0.3]},
    ],
)

# db.set_document_embeddings(
#     collection_id,
#     "document-2",
#     [
#         {"start": 0, "end": 100, "vectors": [0.1, 0.2, 0.3, 0.4]},
#         {"start": 100, "end": 200, "vectors": [0.5, 0.5, 0.2, 0.3]},
#     ],
# )

collection_embeddings = db.scan_embeddings(collection_id)
# print(collection_embeddings)


# print(db.search_collection(collection_id, [0.1, 0.2, 0.3, 0.4], 5))


print(db.search_collection(collection_id, [1, 1, 1, 1], 5))

db.close()
db.destroy()
