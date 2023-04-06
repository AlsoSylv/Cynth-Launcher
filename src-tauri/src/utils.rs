use std::{
    env::consts::OS,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use futures::StreamExt;
use tauri::Window;

use crate::{
    errors::Error,
    json::{AssetMap, Library},
};

const ASSET_BASE_URL: &str = "https://resources.download.minecraft.net";

pub async fn write_assets(
    client: reqwest::Client,
    index_url: String,
    window: Window,
) -> Result<(), Error> {
    let index_name = index_url.split('/').collect::<Vec<&str>>();
    let index_name = index_name.last().unwrap();
    let index_name = &format!("./assets/indexes/{}", index_name);

    if !Path::new(index_name).exists() {
        let bytes_json = client
            .get(index_url)
            .send()
            .await
            .map_err(Error::Request)?
            .bytes()
            .await
            .map_err(Error::Request)?;

        tokio::fs::create_dir_all("./assets/indexes")
            .await
            .map_err(Error::Io)?;
        let index_json = tokio::fs::write(index_name, bytes_json);
        let objs = tokio::fs::create_dir_all("./assets/objects");
        let (objs, index_json) = futures::join!(objs, index_json);
        objs.map_err(Error::Io)?;
        index_json.map_err(Error::Io)?;
    }

    let bytes_json = fs::read(index_name).map_err(Error::Io)?; // TODO: Replacce this with the real name
    let json: AssetMap = serde_json::from_slice(&bytes_json).unwrap();

    let mut paths = Vec::with_capacity(json.capacity());

    let objects = json["objects"].clone();

    let mut requests = Vec::with_capacity(json.capacity());

    objects.iter().for_each(|(_, asset)| {
        let hash = &asset.hash;
        let mut first_two = hash.clone();
        first_two.replace_range(2.., "");

        let dir = format!("./assets/objects/{}", first_two);
        if !Path::new(&dir).exists() {
            let _ = fs::create_dir(&dir);
        }

        let file = format!("{}/{}", dir, hash);
        let path = Path::new(&file);

        if !path.exists() {
            paths.push(file);
            let url = format!("{}/{}/{}", ASSET_BASE_URL, first_two, hash);
            requests.push(url);
        }
    });

    if requests.is_empty() {
        return Ok(());
    }

    let val = Arc::new(AtomicU32::new(0));

    let fetches = futures::stream::iter(requests.iter().enumerate().map(|(pos, url)| {
        let len = requests.len() as f32;
        let val = val.clone();
        let paths = paths.clone();
        let window = window.clone();
        async move {
            match reqwest::get(url).await {
                Ok(res) => match res.bytes().await {
                    Ok(buf) => {
                        let path = paths[pos].clone();
                        let val = val.fetch_add(1, Ordering::Relaxed) as f32;
                        window
                            .emit("asset_download_progress", &format!("{}", val / len))
                            .unwrap();
                        tokio::fs::write(path, buf).await.map_err(Error::Io)
                    }
                    Err(err) => Err(Error::Request(err)),
                },
                Err(err) => Err(Error::Request(err)),
            }
        }
    }))
    .buffer_unordered(16)
    .boxed()
    .collect::<Vec<Result<(), Error>>>();

    let x = fetches.await;

    for val in x {
        val?
    }

    Ok(())
}

pub async fn download_libraries(
    init_libraries: &Vec<Library>,
    client: &reqwest::Client,
    window: Window,
) -> Result<Vec<String>, Error> {
    let mut sha1 = sha1_smol::Sha1::new();

    let mut requests = Vec::with_capacity(init_libraries.len());
    let mut paths = Vec::with_capacity(init_libraries.len());
    let mut needs_download = Vec::with_capacity(init_libraries.len());

    for library in init_libraries {
        let artifact = &library.downloads.artifact;
        let jar = format!("./libraries/{}", &artifact.path);
        if Path::new(&jar).exists() {
            let existing_jar = fs::read(&jar).unwrap();
            sha1.update(&existing_jar);
            let result = sha1.digest().to_string();
            if !result.eq(&artifact.sha1) {
                panic!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
            }

            sha1.reset();

            paths.push(jar);
        } else if let Some(rules) = &library.rules {
            for rule in rules {
                let os_name = rule.os.name.clone();
                if os_name.unwrap().eq(OS) {
                    // let request = client.get(artifact.url.clone()).send();
                    requests.push(artifact.url.clone());
                    paths.push(jar.clone());
                    needs_download.push(jar.clone());
                }
            }
        } else {
            requests.push(artifact.url.clone());
            paths.push(jar.clone());
            needs_download.push(jar);
        }
    }

    let val = Arc::new(AtomicU32::new(0));

    let fetches = futures::stream::iter(requests.iter().enumerate().map(|(pos, url)| {
        let len = requests.len() as f32;
        let paths = needs_download.clone();
        let val = val.clone();
        let client = client.clone();
        let window = window.clone();
        async move {
            match client.get(url).send().await {
                Ok(res) => {
                    let libary_name = res
                        .url()
                        .path_segments()
                        .and_then(|segment| segment.last())
                        .unwrap()
                        .to_owned();
                    match res.bytes().await {
                        Ok(buf) => {
                            let jar_name = paths[pos].clone();
                            let path = jar_name.strip_suffix(&libary_name).unwrap();
                            let _ = tokio::fs::create_dir_all(path).await;
                            let val = val.fetch_add(1, Ordering::Relaxed) as f32;
                            window
                                .emit("library_download_progress", &format!("{}", val / len))
                                .unwrap();
                            tokio::fs::write(jar_name, buf).await.map_err(Error::Io)
                        }
                        Err(err) => Err(Error::Request(err)),
                    }
                }
                Err(err) => Err(Error::Request(err)),
            }
        }
    }))
    .buffer_unordered(16)
    .boxed()
    .collect::<Vec<Result<(), Error>>>();

    let x = fetches.await;

    for y in x {
        y?
    }

    Ok(paths)
}
