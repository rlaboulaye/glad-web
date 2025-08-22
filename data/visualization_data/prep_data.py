import json

import numpy as np
import pandas as pd

EXCLUDE = set([])

# Define Paths
demo_file = "demographic.txt"
igsr_file = "igsr_samples.tsv"
ids_file = "ids.txt"
pca_file = "pca.10dims.tsv"
ibd_file = "ibd.tsv"
communities_map_file = "communities.map.tsv"

# Read PCA
pca = pd.read_csv(pca_file, sep="\t", header=None)
ids = pd.read_csv(ids_file, header=None)[0].tolist()
pca["id"] = ids  # align IDs to PCA rows

# Load demographics
demo = pd.read_csv(demo_file, sep="\t", header=0)

# Load IGSR
igsr = pd.read_csv(igsr_file, sep="\t", header=0, index_col=0)
igsr_ids = set(igsr.index.tolist())

# Merge PCA and demographics
merged = pd.merge(pca, demo, how="left", left_on="id", right_on="ID")

# Load IBD Communities
ibd = pd.read_csv(ibd_file, sep="\t", header=0)
ibd_communities_map = pd.read_csv(communities_map_file, sep="\t", header=None, index_col=0)

# We use PHS, Project, Country, Region_label, Sex, Ethnicity, GLAD_Status from demographic file as truth
ibd.drop(columns = ["PHS", "Project", "Country", "Region_label", "Sex", "Ethnicity", "GLAD_Status"], inplace=True)

# Convert IBD Communities to naming based on community size - matches published names
ibd_communities_map.loc[-1] = None
communities = set(ibd_communities_map.index.tolist())
ibd["Membership"] = ibd_communities_map.loc[ibd.Membership.apply(lambda x: x if x in communities else -1)][1].to_numpy()

# Merge PCA and IBD Communities
merged = pd.merge(merged, ibd, how="left", left_on="id", right_on="Sample")

# Format records
records = []
for _, row in merged.iterrows():
    record = {
        "id": row["id"],
        "pc": [row[i] for i in range(10)],
        "phs": row.get("PHS", None),
        "country": row.get("Country", None),
        "region": row.get("Region_label", None),
        "sex": row.get("Sex", None),
        "ethnicity": row.get("Ethnicity", None),
        "ethnicity_source": row.get("GLAD_Status", None),
        "ibd_matrix_index": row.get("Vcf_ID", None),
        "ibd_community": row.get("Membership", None),
        "1000_genomes": None if row["id"] not in igsr_ids else igsr.loc[row["id"]]["Population code"],
    }
    if record["ibd_matrix_index"] and not np.isnan(record["ibd_matrix_index"]):
        record["ibd_matrix_index"] = int(record["ibd_matrix_index"])
    if record["ethnicity_source"] == "self_described":
        record["ethnicity_source"] = "survey_defined"
    if record["phs"] in EXCLUDE:
        continue
    records.append(record)


# Replace NaNs with None
def nan_to_none(o):
    if isinstance(o, float) and np.isnan(o):
        return None
    if isinstance(o, list):
        return [nan_to_none(i) for i in o]
    if isinstance(o, dict):
        return {k: nan_to_none(v) for k, v in o.items()}
    return o


records_fixed = nan_to_none(records)

# Output to JSON
with open("sample_metadata.json", "w") as f:
    json.dump(records_fixed, f, indent=2)

