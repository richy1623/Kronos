mod utils;
use std::fs;
use utils::app_settings::AppSettings;


fn main() {
    let mut app_settings = AppSettings::new();
    print!("\nUsername is  {}\n",app_settings.get_username());
    print!("Api key is {}\n",app_settings.get_api_key());
}
