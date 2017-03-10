""""""

from argparse import ArgumentParser, FileType
import json
import urllib.parse
import urllib.request

ap = ArgumentParser()
ap.add_argument("-c", "--host")
ap.add_argument("-p", "--passkey")
ap.add_argument("-f", "--source_file", type=FileType("r"))
args = ap.parse_args()

data_dict={
    "passkey": args.passkey,
    "data": args.source_file.read()
}

req = urllib.request.Request("http://{}/address/update_source".format(args.host),
                             data=urllib.parse.urlencode(data_dict).encode())

response = urllib.request.urlopen(req)
print(response, response.read())
