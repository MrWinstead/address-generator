# IPv4 Address Generator

This REST service is designed to be part of a test system
wherein one needs to generate IP addresses which will
reverse-resolve (using something like GeoIP) to a specific
country.

Address Generator is not efficient - it is simply designed to
quickly spit out addresses for a few countries. Since it
pre-populates its underlying data structures upon first request
for a country, it can easily use upwards of 30GB of memory - making
swap space especially important. Additionally, loading all of the
addresses for the "us" country code alone will cause the application
to use 16GB of memory. In the future, I'll improve this, but for now
it works for what I need.

# Data Source

This generator is designed to use the CSV files which may be found at
(Nirsoft's Website)[http://www.nirsoft.net/countryip/] with column headers.
an contrived example of this would be:

```text
country,ip_start,ip_end,total_ips,last_updated,telco
gb,2.2.2.0,2.2.2.255,256,12/31/99,"Her Majesty's telco"
us,1.1.1.0,1.1.1.4,4,11/1/99,test_telco1
de,3.2.0.0,3.2.255.255,65536,1/2/89,
```

# Running

```bash
$ cargo run -- --source country_ip.csv
```

# API

The REST service exposes two endpoints

## ```/addresses```

This endpoint takes no arguments and will generate an IP address which is in
the source CSV file.

## ```/address/<countrycode>```

This endpoint takes the ```countrycode``` argument as a two-letter country code such
as "gb" or "us". Upon first access to a ```countrycode```, the service will generate
all addresses which could be available for the country, so it could take a
while; subsequent accesses to the country code will only be a hashmap and table
lookup's worth of time.

If the ```countrycode``` is unknown, this will return a ```404 Not Found```.
