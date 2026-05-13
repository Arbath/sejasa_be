#[test]
fn calculate(){
    let lat1 = -7.783964001537162;
    let lon1 = 110.39444866671627;
    let lat2 = -7.7611383777145795;
    let lon2 = 110.5321657296927;
    let jarak = calculate_distance_in_meters(lat1, lon1, lat2, lon2);
    let km = 1000.0;
    println!("Jarak : {}m", jarak);
    println!("Jarak : {}km", jarak/km);
}


fn calculate_distance_in_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    // Jari-jari rata-rata bumi dalam meter
    let earth_radius_meters = 6_371_000.0;

    // Ubah derajat ke radian
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    // Kalkulasi Haversine
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_meters * c
}