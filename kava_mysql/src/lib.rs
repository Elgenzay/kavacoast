use mysql::{Pool, PooledConn};

pub fn get_mysql_connection() -> Result<PooledConn, String> {
	let pass = std::env::var("MYSQL_PASS").expect("Missing environment variable: MYSQL_PASS");
	let url = format!("mysql://kava:{}@localhost:3306/kava", pass);
	let pool = match Pool::new(url.as_str()) {
		Ok(v) => v,
		Err(e) => return Err(e.to_string()),
	};
	match pool.get_conn() {
		Ok(v) => Ok(v),
		Err(e) => Err(e.to_string()),
	}
}
