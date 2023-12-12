# cloudflare-ddns-rust

This repo aims to reconstrust the original [cloudflare-ddns](https://github.com/un-lock-able/cloudflare-ddns) using rust. This is a simple ddns client used for Cloudflare. 

You may first want to read about how to [configure](#customize-the-settings) it, [learn about command line arguments](#command-line-arguments) and you may want to [run it periodically](#periodially-run-the-script-using-crontab).

## Customize the settings
There is an example configure file named `settings.example.json`. I hope it would be clear enough for you to create your own `settings.json` file.

If you need a more detailed information on the schema of the json, below are detailed discriptions of the schema of the config:

### Base object
This is the base object of the config file.
| Field Name       | Required | Description                                                                                                                                                                                  |
| :--------------- | :------: | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `getIPUrls`      |   Yes    | An [object](#config-for-urls-for-retriving-public-ip) storing the api urls for retriving the current server's public ip address.                                                                        |
| `domainSettings` |   Yes    | An array of [singleDomainSettings](#config-for-every-single-domain) for every domain in cloudflare. Note that you have to create seperate config for AAAA and A records for the same domain. |

### Config for urls for retriving public ip
This is the object storing the urls the script will use to determine the server's public ip address. 

Back to parent: [Base config object](#base-object).

| Field Name | Required | Description                                                                                                                               |
| :--------- | :------: | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `IPv4`     |   Yes    | A string of the url that is used for acquire the IPv4 public address of the server. There are two usable urls in `settings.example.json'. |
| `IPv6`     |   Yes    | A string of the url that is used for acquire the IPv6 public address of the server. There are two usable urls in `settings.example.json'. |

### Config for every single domain
This is the config for a 'domain' that you add to your cloudflare account, i.e. you have a zone ID for it.

**Note: One 'config for a domain' can only deal with one kind of record (A or AAAA), so if you wish to enable DDNS for both A and AAAA record for the same domain, create two config for the same domain name.**

Back to parent: [Base config object](#base-object).

| Field Name        | Required | Description                                                                                                                                        |
| :---------------- | :------: | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `enabled`         |   Yes    | A boolean value to enable (`true`) or disable (`false`) the config.                                                                                |
| `domainName`      |   Yes    | A string of the domain name you want to enable DDNS for (the name displayed in cloudflare).                                                        |
| `zone_id`         |   Yes    | A string of the zone ID of the correspoding domain you wish to enable DDNS for.                                                                    |
| `apiToken`        |   Yes    | A string of the apiToken for accessing the cloudflare api. Ensure the apiToken has the permission to edit DNS record for the corresponding domain. |
| `recordType`      |   Yes    | A string of `"A"` or `"AAAA"`, standing for ipv4 and ipv6, respectively.                                                                           |
| `createNewRecord` |   Yes    | A boolean controlling whether to create a new DNS record pointing to the server's address when no DNS record exists for a subdomain in the config.    |
| `subdomains`      |   Yes    | An array of [subdomain settings](#config-for-every-subdomain). Listing all the subdomains that need DDNS and their settings.                       |

### Config for every subdomain
This is the config for every subdomain under a domain name. Only the `name` field is required and others are some extra options that may be of some help.

Back to parent: [Single domain config](#config-for-every-single-domain).

| Field Name    | Required | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
|---------------|:--------:|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `name`        |    Yes   | A string that is the name of the subdomain. Subdomain names will be concatenated with domain name to create a full domain name. For example, `test` with domain name of `example.com` will enable DDNS for `test.example.com`.<br><br>If your domain name is `example.com` and you want to enable DDNS for it, use `""` or `"@"` here.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `ttl`         |    No    | A positive integer ranged from 60 to 86400, the Time To Live of the record in seconds. Set 1 for 'automatic'. <br><br>*Default is 1.*                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `proxied`     |    No    | A boolean specifying whether the request to this domain is being proxied by cloudflare. If you wish to make request other than http and https e.g. ssh or remote desktop, generally this should be false.<br><br>*Default is true.*                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `interfaceID` |    No    | **Only takes effect when recordType is AAAA**<br><br>A string specifying the last 64 bit of the ip needed to be updated into the DNS record. This is useful when you want to enable DDNS for a device that is on the same network of the server, but cannot run this script or you want to specify another ip for receiving request. The string should be a valid ipv6 address, for example `::39:c5bb`, and the script will overwrite the last 64bit of the ip updated with DDNS using the last 64bit of content specified in this field.<br><br>For example, the ip the server got is `2001:4860:4860::8888` and `interfaceID` is set to `::39:c5bb`, then the actual ip written in the DNS record with be `2001:4860:4860::39:c5bb`.<br><br>*When not provided, the default behavior is to use the ip returned by the getIPurl api. |

## Command line arguments

This script does not accept config from command line arguments. Please be sure to [configure your DDNS](#configuring-the-settings) before you run the application.

`cloudflare-ddns-rust --help` gives
```
Usage: cloudflare-ddns-rust [OPTIONS] --config <CONFIG>

Options:
      --debug                
  -c, --config <CONFIG>      
      --log-file <LOG_FILE>  Path to the log file. Will create all the parent directory if none exist. Defaults to ddnslog.log file in the same directory as the excutable.
  -n <THREAD_NUMBER>         The number of threads used to update the domains. Default to 4. [default: 4]
  -h, --help                 Print help
  -V, --version              Print version
```

Among all of these options, the most important one would be `-c` or `--config` for specifying the location of the config file. This is the only argument that is required.

## Periodially run the script using crontab

On windows, one can easily configure the system to run the script periodically using task scheduler.

On linux, we can configure crontab to periodically run the script for us. Type 
```shell
crontab -e
```
will open crontab's config file with your default editor. Then add
```
*/10 * * * * /absolute/path/to/executable -c /absolute/path/to/config/file
```
to the end of the file. This will run the script every 10 minutes.

## Build the project
Simply run 
```shell
cargo build --release
```
and cargo should auotomatically download all dependencies the build the project.
