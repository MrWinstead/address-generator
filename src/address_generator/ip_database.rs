
use byteorder::{BigEndian, ReadBytesExt};

use csv::Reader;

use rand;
use rand::Rng;

use std::collections::HashMap;
use std::error::Error;
use std::io::Cursor;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Debug)]
struct IPGeoEntry {
    country_code: String,
    ip_start: Ipv4Addr,
    ip_end: Ipv4Addr,
    total_ips: usize,
}


#[derive(Debug, RustcDecodable)]
struct RawIPGeoEntry {
    country_code: String,
    ip_start: String,  // I wish I could use the IPv4Addr type, but rustc_serialize doesn't like it
    ip_end: String,
    total_ips: usize,
    last_updated: String,
    owning_telco: Option<String>,
}


#[derive(Debug)]
pub struct IPGeoDatabase {
    generated_ips: HashMap<String, Vec<Ipv4Addr>>,
    ipgeo_data: HashMap<String, Vec<IPGeoEntry>>,
}

fn ip2u32(address: &Ipv4Addr) -> u32 {
    let octets = address.octets();
    let mut rdr = Cursor::new(octets);
    rdr.read_u32::<BigEndian>().unwrap()
}

impl IPGeoDatabase {

    pub fn new() -> IPGeoDatabase {
        IPGeoDatabase{
            generated_ips: HashMap::new(),
            ipgeo_data: HashMap::new(),
        }
    }

    pub fn set_underlying_data(&mut self, source: String) -> Result<bool, String> {
        let mut csv_reader = Reader::from_string(source.trim());

        for record in csv_reader.decode() {
            if record.is_err() {
                let record_str = format!("{:?}", record);
                return Err(format!("Could not parse record '{}': {:?}",
                                   record_str,
                                   record.unwrap_err().description()));
            } else {
                let record: RawIPGeoEntry = record.unwrap();

                let store_result = self.store_single_record(record);

                if store_result.is_err() {
                    return Err(store_result.unwrap_err());
                }
            }
        }
        Ok(true)
    }

    fn store_single_record(&mut self, record: RawIPGeoEntry) -> Result<bool, String> {
        let ip_start_res = Ipv4Addr::from_str(&(record.ip_start));
        let ip_end_res = Ipv4Addr::from_str(&(record.ip_end));
        if !ip_start_res.is_ok() {
            return Err(format!("Could not parse ip start: {}",
                               ip_start_res.unwrap_err().description()))
        }else if !ip_end_res.is_ok() {
            return Err(format!("Could not parse ip end: {}",
                               ip_start_res.unwrap_err().description()))
        }

        let ip_start = ip_start_res.unwrap();
        let ip_end = ip_end_res.unwrap();

        let entry = IPGeoEntry {
            country_code: record.country_code.trim().to_string(),
            ip_start: ip_start,
            ip_end: ip_end,
            total_ips: record.total_ips,
        };
        if ! self.ipgeo_data.contains_key(&entry.country_code) {
            let new_vec: Vec<IPGeoEntry> = Vec::new();
            self.ipgeo_data.insert(entry.country_code.clone(), new_vec);
        }

        let mut entry_list = self.ipgeo_data.get_mut(&entry.country_code).unwrap();
        entry_list.push(entry);

        Ok(true)
    }

    fn populate_generated_ips(&mut self, country_code: &String) {
        let all_ip_entries: &Vec<IPGeoEntry> = self.ipgeo_data.get(country_code).unwrap();
        let mut generated_ip_list: Vec<Ipv4Addr> = Vec::new();

        for entry in all_ip_entries {
            let mut current_address = ip2u32(&entry.ip_start) + 1; // skip the network address
            let end_address = ip2u32(&entry.ip_end) - 1; // subtract the broadcast address

            while current_address <= end_address {
                let addr = Ipv4Addr::from(current_address);
                generated_ip_list.push(addr);
                current_address += 1;
            }
        }

        self.generated_ips.insert(country_code.clone(), generated_ip_list);
    }


    pub fn get_random_address(&mut self, country_code: &String) -> Result<Ipv4Addr, String> {
        let mut rng = rand::thread_rng();
        if ! self.ipgeo_data.contains_key(country_code) {
            return Err(format!("unknown country code '{:?}'", country_code))
        }

        if ! self.generated_ips.contains_key(country_code) {
            self.populate_generated_ips(country_code);
        }

        let ip_list = self.generated_ips.get(country_code).unwrap();
        let selected_ip = ip_list[rng.gen::<usize>() % ip_list.len()];
        Ok(selected_ip)
    }

    pub fn get_random_country_code(&self) -> Option<String> {
        let mut rng = rand::thread_rng();
        if self.ipgeo_data.len() > 0 {
            let mut keys = self.ipgeo_data.keys();
            let random_index = rng.gen::<usize>() % keys.len();
            let pulled_country_code = keys.nth(random_index).unwrap();
            Some(pulled_country_code.clone())
        }else {
            None
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    fn get_default_instance() -> IPGeoDatabase {
        let mut instance = IPGeoDatabase::new();
        let res = instance.set_underlying_data(TEST_CSV.to_string());
        assert_eq!(res, Ok(true));
        instance
    }

    #[test]
    fn test_instantiation() {
        let instance = IPGeoDatabase::new();
        assert_eq!(instance.generated_ips.len(), 0);
    }

    #[test]
    fn test_set_underlying_data() {
        let instance = get_default_instance();
        assert_eq!(instance.ipgeo_data.len(), 3);
    }

    #[test]
    fn test_generate_addresses() {
        let mut instance = get_default_instance();

        let random_address = instance.get_random_address(&"gb".to_string());
        assert!(random_address.is_ok());
    }

    #[test]
    fn test_random_country() {
        let instance = get_default_instance();
        let random_code_opt = instance.get_random_country_code();
        if random_code_opt.is_none() {
            assert!(false, "Received no country code");
        }
        let random_code = random_code_opt.unwrap();

        if random_code == "gb" || random_code == "de" || random_code == "us" {
            assert!(true);
        }else {
            assert!(false, format!("Received unknown country code '{}'", random_code));
        }

    }

    static TEST_CSV: &str = r#"
        country,ip_start,ip_end,total_ips,last_updated,telco
        gb,2.2.2.0,2.2.2.255,256,12/31/99,"Her Majesty's telco"
        us,1.1.1.0,1.1.1.4,4,12/31/99,test_telco1
        de,3.2.0.0,3.2.255.255,65536,12/31/99,
        "#;
}
