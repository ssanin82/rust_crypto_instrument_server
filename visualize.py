#!/usr/bin/env python3

import sqlite3
import pandas as pd
import http.server
import socketserver
import threading
from pathlib import Path

# -----------------------
# Config
# -----------------------
DB_PATH = "crypto_refdata.db"
PORT = 5555
HTML_FILE = "reference_data.html"
PAGE_TITLE = "Crypto Reference Data"

# -----------------------
# Load data from SQLite
# -----------------------
conn = sqlite3.connect(DB_PATH)

query = """
SELECT
    id,
    product_type,
    exchange,
    symbol,
    tick_size,
    lot_size,
    updated_at
FROM reference_data
ORDER BY id ASC
"""

df = pd.read_sql_query(query, conn)
conn.close()

# -----------------------
# Generate HTML (DataTables)
# -----------------------
table_headers = "".join(f"<th>{col}</th>" for col in df.columns)

table_rows = "\n".join(
    "<tr>" + "".join(f"<td>{value}</td>" for value in row) + "</tr>"
    for row in df.itertuples(index=False)
)

html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{PAGE_TITLE}</title>

    <!-- DataTables + jQuery -->
    <script src="https://code.jquery.com/jquery-3.7.1.min.js"></script>
    <script src="https://cdn.datatables.net/1.13.8/js/jquery.dataTables.min.js"></script>

    <!-- DataTables Dark Theme -->
    <link rel="stylesheet" href="https://cdn.datatables.net/1.13.8/css/jquery.dataTables.min.css">

    <style>
        body {{
            background-color: #020617;
            color: #e5e7eb;
            font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
            margin: 40px;
        }}

        h1 {{
            text-align: center;
            margin-bottom: 30px;
            font-size: 32px;
        }}

        table.dataTable {{
            background-color: #020617;
        }}

        table.dataTable thead th {{
            background-color: #1f2937;
            color: white;
        }}

        table.dataTable tbody tr {{
            background-color: #111827;
        }}

        table.dataTable tbody tr:hover {{
            background-color: #1f2937;
        }}

        .dataTables_wrapper .dataTables_filter input,
        .dataTables_wrapper .dataTables_length select {{
            background-color: #020617;
            color: #e5e7eb;
            border: 1px solid #374151;
        }}

        .dataTables_wrapper .dataTables_info,
        .dataTables_wrapper .dataTables_paginate {{
            color: #9ca3af;
        }}
    </style>
</head>
<body>

<h1>{PAGE_TITLE}</h1>

<table id="refdata" class="display" style="width:100%">
    <thead>
        <tr>
            {table_headers}
        </tr>
    </thead>
    <tbody>
        {table_rows}
    </tbody>
</table>

<script>
$(document).ready(function() {{
    $('#refdata').DataTable({{
        order: [[0, 'asc']],   // default sort by id
        pageLength: 25,
        lengthMenu: [10, 25, 50, 100],
        stateSave: true
    }});
}});
</script>

</body>
</html>
"""

Path(HTML_FILE).write_text(html, encoding="utf-8")

# -----------------------
# Serve directory over HTTP
# -----------------------
class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        pass  # silence logs

def serve():
    with socketserver.TCPServer(("", PORT), QuietHandler) as httpd:
        print(f"Serving at http://localhost:{PORT}/{HTML_FILE}")
        httpd.serve_forever()

threading.Thread(target=serve, daemon=True).start()

input("Press Enter to stop server...\n")
