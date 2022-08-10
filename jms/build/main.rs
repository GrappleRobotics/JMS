mod gen_schema;

// use std::env;
// use std::fs;

fn main() -> anyhow::Result<()> {
  // let outdir = match env::var_os("OUT_DIR") {
  //   Some(outdir) => outdir,
  //   None => anyhow::bail!("OUT_DIR not set!")
  // };

  // fs::create_dir_all(&outdir)?;

  // gen_schema::generate_schema(&outdir);

  Ok(())
}