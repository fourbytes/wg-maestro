#!/usr/bin/python
'''
Usage::

    ./decoder.py [module] [data_file]

Sample::

    ./decoder.py pyroute2.netlink.rtnl.tcmsg.tcmsg ./sample_packet_01.data
    ./decoder.py pyroute2.netlink.nl80211.nl80211cmd ./nl80211.data

Module is a name within rtnl hierarchy. File should be a
binary data in the escaped string format (see samples).
'''
import sys
from io import StringIO
from pprint import pprint
from importlib import import_module
from pyroute2.common import load_dump
from pyroute2.common import hexdump
from pyroute2.netlink.generic.wireguard import wgmsg as met

if __name__ == "__main__":
    with open(sys.argv[1], 'r') as f:
        for line in f.readlines():
            try:
                data = load_dump(StringIO(line))
                offset = 0
                inbox = []
                while offset < len(data):
                    msg = met(data[offset:])
                    msg.decode()
                    print(hexdump(msg.data))
                    pprint(msg)
                    print('.'*40)
                    offset += msg['header']['length']
            except Exception as e:
                pprint(e)
