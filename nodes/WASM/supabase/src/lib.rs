use space_lib::Supabase;
use std::env::var;

#[no_mangle]
fn main(bucket: &String, select: &String) -> Box<String> {
    let supabase_url = var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let supabase_anon_key = var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY must be set");
    let client = Supabase::new(supabase_url).insert_header("apikey", supabase_anon_key);
    let response = client
        .from(bucket)
        .select(select)
        .execute()
        .unwrap()
        .into_string()
        .unwrap();
    Box::new(response)
}
