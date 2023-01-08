use mysql::{Pool, PooledConn};

pub fn get_mysql_connection() -> Result<PooledConn, String> {
	let pass = std::env::var("MYSQL_PASS").expect("Missing environment variable: MYSQL_PASS");
	let url: &str =
		&(String::from("mysql://kava:") + &pass + &String::from("@localhost:3306/kava"))[..];
	let pool = match Pool::new(url) {
		Ok(v) => v,
		Err(e) => return Err(e.to_string()),
	};
	match pool.get_conn() {
		Ok(v) => Ok(v),
		Err(e) => Err(e.to_string()),
	}
}
