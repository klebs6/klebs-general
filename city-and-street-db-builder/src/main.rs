use usa_city_and_street_db_builder::*;
use usa::*;
use tracing_setup::*;
use tracing::{warn,info};
use std::path::PathBuf;
use std::sync::{Arc,Mutex};

#[tokio::main]
async fn main() -> Result<(),UsaCityAndStreetDbBuilderError> {

    configure_tracing();

    let mock = false;

    match mock {
        true  => main_mock().await,
        false => main_no_mock().await,
    }
}

async fn build_dmv_regions(db: &mut Database) 
    -> Result<(),UsaCityAndStreetDbBuilderError> 
{
    info!("building DMV regions from PBF data");

    // Once mock is tested, we could run the real build:
    let regions: Vec<USRegion> = vec![
        USRegion::UnitedState(UnitedState::Maryland),
        USRegion::UnitedState(UnitedState::Virginia),
        USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia),
    ];

    let target_dir = PathBuf::from("pbf");
    build_regions(&regions, db, &target_dir).await?;

    Ok(())
}

fn validate_mock_address(db: Arc<Mutex<Database>>) -> Result<(),UsaCityAndStreetDbBuilderError> {

    // Queries:
    let da      = DataAccess::with_db(db.clone());

    let address = UsaAddress::mock();

    match address.validate_with(&da) {
        Ok(_) => {
            info!(
                "Address is valid {:#?} -> true", 
                address
            );
        },
        Err(e) => {
            warn!(
                "Address is invalid {:#?} -> false\nerr={:#?}", 
                address,
                e
            );
        }
    }

    Ok(())
}

async fn main_no_mock() -> Result<(),UsaCityAndStreetDbBuilderError> {

    let db  = Database::open(&PathBuf::from("rocksdb_us"))?;

    match db.lock() {
        Ok(mut db) => build_dmv_regions(&mut *db).await?,
        Err(_) => panic!("could not get db lock!"),
    }

    validate_mock_address(db.clone());

    Ok(())
}

async fn main_mock() -> Result<(),UsaCityAndStreetDbBuilderError> {

    let db  = Database::open(&PathBuf::from("rocksdb_mock"))?;

    let region = USRegion::UnitedState(UnitedState::Maryland);

    // For demonstration, let's use mock data:
    let mock_regional_records = RegionalRecords::mock_for_region(&region);

    match db.lock() {
        Ok(mut db) => {
            mock_regional_records.write_to_storage(&mut db)?;
        },
        Err(_) => panic!("could not get DB lock!"),
    }

    validate_mock_address(db.clone());

    Ok(())
}
