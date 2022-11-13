use std::io::Write;

use actix_multipart::Multipart;
use actix_web::web;
use futures::{StreamExt, TryStreamExt};

pub async fn save_file(mut payload: Multipart, file_path: String) -> Option<bool> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        //let filename = content_type.get_filename().unwrap();
        let filepath = file_path.clone();

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f))
                .await
                .unwrap();
        }
    }

    Some(true)
}
