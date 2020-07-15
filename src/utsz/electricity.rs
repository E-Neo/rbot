use prettytable::{Cell, Table};
use reqwest::Client;
use serde::Serialize;
use soup::prelude::*;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const ROOM_FILL_LOG_VIEW_URL: &str = "http://10.64.1.18/webSelect/roomFillLogView.do";
const WELCOM2_URL: &str = "http://10.64.1.18/webSelect/welcome2.jsp";

#[derive(Serialize)]
struct Room<'a> {
    #[serde(rename = "buildingId")]
    building_id: &'a str,
    #[serde(rename = "roomName")]
    room_name: &'a str,
}

impl<'a> Room<'a> {
    fn new(building_id: &'a str, room_name: &'a str) -> Self {
        Self {
            building_id,
            room_name,
        }
    }
}

pub struct Info {
    top_up_infos: Vec<Vec<String>>,
    charge_infos: Vec<Vec<String>>,
}

impl Info {
    fn new(top_up_infos: Vec<Vec<String>>, charge_infos: Vec<Vec<String>>) -> Self {
        Self {
            top_up_infos,
            charge_infos,
        }
    }
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "充值记录\n{}电量使用记录\n{}",
            Table::init(
                self.top_up_infos
                    .iter()
                    .map(|row| row.iter().map(|cell| Cell::new(cell)).collect())
                    .collect()
            ),
            Table::init(
                self.charge_infos
                    .iter()
                    .map(|row| row.iter().map(|cell| Cell::new(cell)).collect())
                    .collect()
            )
        )
    }
}

pub async fn electricity(building_id: &str, room_name: &str) -> reqwest::Result<Info> {
    let client = Client::builder()
        .cookie_store(true)
        .user_agent(USER_AGENT)
        .build()?;
    let res = client
        .post(ROOM_FILL_LOG_VIEW_URL)
        .form(&Room::new(building_id, room_name))
        .send()
        .await?;
    assert_eq!(res.status(), 200, "ROOM_FILL_LOG_VIEW_URL");
    let res = client.get(WELCOM2_URL).send().await?;
    assert_eq!(res.status(), 200, "WELCOM2_URL");
    let soup = Soup::new(&res.text().await?);
    Ok(Info::new(
        extract_top_up_infos(&soup),
        extract_charge_infos(&soup),
    ))
}

fn extract_top_up_infos(soup: &Soup) -> Vec<Vec<String>> {
    soup.tag("div")
        .attr("id", "fillDiv")
        .find()
        .expect("Couldn't find tag div with id \"fillDiv\"")
        .tag("tr")
        .find_all()
        .map(|tr| {
            tr.tag("td")
                .find_all()
                .skip(1)
                .map(|td| td.text())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn extract_charge_infos(soup: &Soup) -> Vec<Vec<String>> {
    soup.tag("div")
        .attr("id", "usedEleDiv")
        .find()
        .expect("Couldn't find tag div with id \"usedEleDiv\"")
        .tag("tr")
        .find_all()
        .map(|tr| {
            tr.tag("td")
                .find_all()
                .skip(2)
                .map(|td| td.text())
                .collect::<Vec<_>>()
        })
        .collect()
}
