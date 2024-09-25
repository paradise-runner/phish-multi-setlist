use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;
use wasm_bindgen::JsValue;

#[derive(Debug, Deserialize, Serialize)]
struct Show {
    showid: String,
    showdate: String,
    permalink: String,
    showyear: String,
    uniqueid: String,
    meta: String,
    reviews: String,
    exclude: String,
    setlistnotes: String,
    soundcheck: String,
    songid: String,
    position: String,
    transition: String,
    footnote: String,
    set: String,
    isjam: String,
    isreprise: String,
    isjamchart: String,
    jamchart_description: String,
    tracktime: String,
    gap: String,
    tourid: String,
    tourname: String,
    tourwhen: String,
    song: String,
    nickname: String,
    slug: String,
    is_original: String,
    venueid: String,
    venue: String,
    city: String,
    state: String,
    country: String,
    trans_mark: String,
    artistid: String,
    artist_slug: String,
    artist_name: String,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    console_error_panic_hook::set_once();

    // Handle CORS preflight request
    if req.method() == Method::Options {
        return handle_cors_preflight();
    }

    // Parse the URL and extract query parameters
    let url = req.url()?;
    let query_params: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let show_ids: Vec<String> = query_params
        .iter()
        .filter(|(key, _)| key == "showid")
        .map(|(_, value)| value.clone())
        .collect();

    // Get a reference to the D1 database
    let db = env.d1("DB")?;

    // if no show_ids are provided, return an error
    if show_ids.is_empty() {
        return Response::error("No showid query parameter provided", 400);
    }

    // Prepare the SQL query
    let placeholders: Vec<String> = (1..=show_ids.len()).map(|i| format!("?{}", i)).collect();
    let query = format!(
        "SELECT * FROM shows WHERE showid IN ({})",
        placeholders.join(", ")
    );
    
    // Create a map for parameter binding
    let params: Vec<(String, String)> = show_ids.clone().into_iter()
        .enumerate()
        .map(|(i, id)| (format!("{}", i + 1).into(), id.into()))
        .collect();

    // iterate over the params and and cast them to JSValue
    let params: Vec<JsValue> = params.into_iter().map(|(_k, v)| v.into()).collect();
    
    let statement = db.prepare(&query);
    let results = statement.bind(&params)?.all().await?;

    // Parse the results
    let shows: Vec<Show> = results.results::<Show>()?.into_iter().collect();

    // If any show is not found in the database, fetch it from the API
    let mut api_fetched_shows = Vec::new();
    for show_id in &show_ids {
        if !shows.iter().any(|s| &s.showid == show_id) {
            match fetch_from_api(show_id).await {
                Ok(mut new_shows) => {
                    // Insert the new shows into the database
                    for show in &new_shows {
                        insert_show_into_db(&db, show).await?;
                    }
                    api_fetched_shows.append(&mut new_shows);
                }
                Err(e) => {
                    console_log!("Error fetching from API for show_id {}: {:?}", show_id, e);
                }
            }
        }
    }

    // Combine database results and API fetched results
    let all_shows: Vec<Show> = shows
        .into_iter()
        .chain(api_fetched_shows.into_iter())
        .collect();
    

    // Prepare the response JSON
    let response_data = json!({
        "status": 200,
        "statusText": "OK",
        "headers": {
            "content-type": "application/json"
        },
        "body": {
            "data": all_shows
        }
    });

    // Create the response with CORS headers
    let mut response = Response::from_json(&response_data)?;
    add_cors_headers(&mut response);

    Ok(response)
}
async fn fetch_from_api(show_id: &str) -> worker::Result<Vec<Show>> {
    let api_url = format!(
        "https://api.phish.net/v5/setlists/showid/{}.json?apikey=CB157F950763AB53102C",
        show_id
    );

    match reqwest::get(&api_url).await {
        Ok(response) => {
            if response.status().is_success() {
                let api_data = response.text().await.unwrap_or_default();
                let show_data_draw: serde_json::Value = serde_json::from_str(&api_data)?;

                let show_data = &show_data_draw["data"];
                
                if let Some(shows) = show_data.as_array() {
                    let mut fetched_shows = Vec::new();
                    for show in shows {
                        fetched_shows.push(Show {
                            showid: show["showid"].as_str().unwrap_or_default().into(),
                            showdate: show["showdate"].as_str().unwrap_or_default().into(),
                            permalink: show["permalink"].as_str().unwrap_or_default().into(),
                            showyear: show["showyear"].as_str().unwrap_or_default().into(),
                            uniqueid: show["uniqueid"].as_str().unwrap_or_default().into(),
                            meta: show["meta"].as_str().unwrap_or_default().into(),
                            reviews: show["reviews"].as_str().unwrap_or_default().into(),
                            exclude: show["exclude"].as_str().unwrap_or_default().into(),
                            setlistnotes: show["setlistnotes"].as_str().unwrap_or_default().into(),
                            soundcheck: show["soundcheck"].as_str().unwrap_or_default().into(),
                            songid: show["songid"].as_str().unwrap_or_default().into(),
                            position: show["position"].as_str().unwrap_or_default().into(),
                            transition: show["transition"].as_str().unwrap_or_default().into(),
                            footnote: show["footnote"].as_str().unwrap_or_default().into(),
                            set: show["set"].as_str().unwrap_or_default().into(),
                            isjam: show["isjam"].as_str().unwrap_or_default().into(),
                            isreprise: show["isreprise"].as_str().unwrap_or_default().into(),
                            isjamchart: show["isjamchart"].as_str().unwrap_or_default().into(),
                            jamchart_description: show["jamchart_description"].as_str().unwrap_or_default().into(),
                            tracktime: show["tracktime"].as_str().unwrap_or_default().into(),
                            gap: show["gap"].as_str().unwrap_or_default().into(),
                            tourid: show["tourid"].as_str().unwrap_or_default().into(),
                            tourname: show["tourname"].as_str().unwrap_or_default().into(),
                            tourwhen: show["tourwhen"].as_str().unwrap_or_default().into(),
                            song: show["song"].as_str().unwrap_or_default().into(),
                            nickname: show["nickname"].as_str().unwrap_or_default().into(),
                            slug: show["slug"].as_str().unwrap_or_default().into(),
                            is_original: show["is_original"].as_str().unwrap_or_default().into(),
                            venueid: show["venueid"].as_str().unwrap_or_default().into(),
                            venue: show["venue"].as_str().unwrap_or_default().into(),
                            city: show["city"].as_str().unwrap_or_default().into(),
                            state: show["state"].as_str().unwrap_or_default().into(),
                            country: show["country"].as_str().unwrap_or_default().into(),
                            trans_mark: show["trans_mark"].as_str().unwrap_or_default().into(),
                            artistid: show["artistid"].as_str().unwrap_or_default().into(),
                            artist_slug: show["artist_slug"].as_str().unwrap_or_default().into(),
                            artist_name: show["artist_name"].as_str().unwrap_or_default().into(),
                        });
                    }
                    Ok(fetched_shows)
                } else {
                    Err("Invalid API response format".into())
                }
            } else {
                Err(format!("API request failed with status code: {}", response.status()).into())
            }
        }
        Err(e) => {
            Err(format!("API request failed: {:?}", e).into())
        }
    }
}
async fn insert_show_into_db(db: &D1Database, show: &Show) -> worker::Result<()> {
    let query = "INSERT INTO shows (
        showid,
        showdate,
        showyear,
        venue,
        city,
        state,
        country,
        permalink,
        uniqueid,
        meta,
        reviews,
        exclude,
        setlistnotes,
        soundcheck,
        songid,
        position,
        transition,
        footnote,
        \"set\",
        isjam,
        isreprise,
        isjamchart,
        jamchart_description,
        tracktime,
        gap,
        tourid,
        tourname,
        tourwhen,
        song,
        nickname,
        slug,
        is_original,
        venueid,
        city,
        state,
        country,
        trans_mark,
        artistid,
        artist_slug,
        artist_name
    ) VALUES (
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?,
        ?
    )";
    db.prepare(query)
        .bind(&[
            show.showid.as_str().into(),
            show.showdate.as_str().into(),
            show.showyear.as_str().into(),
            show.venue.as_str().into(),
            show.city.as_str().into(),
            show.state.as_str().into(),
            show.country.as_str().into(),
            show.permalink.as_str().into(),
            show.uniqueid.as_str().into(),
            show.meta.as_str().into(),
            show.reviews.as_str().into(),
            show.exclude.as_str().into(),
            show.setlistnotes.as_str().into(),
            show.soundcheck.as_str().into(),
            show.songid.as_str().into(),
            show.position.as_str().into(),
            show.transition.as_str().into(),
            show.footnote.as_str().into(),
            show.set.as_str().into(),
            show.isjam.as_str().into(),
            show.isreprise.as_str().into(),
            show.isjamchart.as_str().into(),
            show.jamchart_description.as_str().into(),
            show.tracktime.as_str().into(),
            show.gap.as_str().into(),
            show.tourid.as_str().into(),
            show.tourname.as_str().into(),
            show.tourwhen.as_str().into(),
            show.song.as_str().into(),
            show.nickname.as_str().into(),
            show.slug.as_str().into(),
            show.is_original.as_str().into(),
            show.venueid.as_str().into(),
            show.city.as_str().into(),
            show.state.as_str().into(),
            show.country.as_str().into(),
            show.trans_mark.as_str().into(),
            show.artistid.as_str().into(),
            show.artist_slug.as_str().into(),
            show.artist_name.as_str().into(),
        ])?
        .run()
        .await?;
    Ok(())
}

fn handle_cors_preflight() -> worker::Result<Response> {
    let mut response = Response::empty()?;
    add_cors_headers(&mut response);
    Ok(response)
}

fn add_cors_headers(response: &mut Response) {
    if let Err(e) = response
        .headers_mut()
        .set("Access-Control-Allow-Origin", "*")
    {
        console_log!("Error setting header: {:?}", e);
    }
    if let Err(e) = response
        .headers_mut()
        .set("Access-Control-Allow-Credentials", "true")
    {
        console_log!("Error setting header: {:?}", e);
    }
    if let Err(e) = response
        .headers_mut()
        .set("Access-Control-Allow-Methods", "GET, OPTIONS")
    {
        console_log!("Error setting header: {:?}", e);
    }
    if let Err(e) = response
        .headers_mut()
        .set("Access-Control-Allow-Headers", "*")
    {
        console_log!("Error setting header: {:?}", e);
    }
}
