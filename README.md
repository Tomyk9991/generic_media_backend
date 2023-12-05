# Image and Video Backend
An image and video Backend server written in Rust 🦀 and Actix_web 🚀

## Current media endpoints

| Name   | Method | Endpoint                  | Description                                                                                                                         | Protected by auth |
|--------|--------|---------------------------|-------------------------------------------------------------------------------------------------------------------------------------|-------------------|
| list   | GET    | /media/{user_name}        | Gets a list with all images from the provided user. Optionally a query with `{limit, offset}` is possible for some pagination logic | YES               |
| upload | POST   | /media/{user_name}        | Upload an file (jpeg, png, mp4) to the `user_name`                                                                                  | YES               |
| file   | GET    | /image/{user_name}/{path} | Gets an individual image                                                                                                            | YES               |


## Current Auth endpoints


| Name              | Method | Endpoint                       | Description                                                                                                                         | Protected by auth |
|-------------------|--------|--------------------------------|-------------------------------------------------------------------------------------------------------------------------------------|-------------------|
| cookie_revalidate | GET    | /authCookie/revalidate         | Checks, if the sended cookie is valid                                                                                               | NO                |
| cookie_auth       | GET    | /authCookie                    | Creates and returns a new cookie, if the provided BasicAuth is a valid `{username, password}` combination                           | NO                |
| logout            | GET    | /logout                        | Sends an empty cookie, which is replaced with the current cookie on the client side. This provides a logout mechanism               | NO                |


## Current User endpoints

| Name            | Method | Endpoint                      | Description                                                          | Protected by auth |
|-----------------|--------|-------------------------------|----------------------------------------------------------------------|-------------------|
| create_user     | Post   | /user                         | Creates a new user by providing `{ username, password, description}` | NO                |
| information     | GET    | /user/{user_name}/information | Get the full `user_name` information `{amount_posts, description}`   | YES               |
| put_information | PUT    | /user/{user_name}/information | Put the full `user_name` information `{description}                  | YES               |
| avatar          | GET    | /user/{user_name}/avatar      | Get the current avatar image as a blob                               | YES               |
| put_avatar      | POST   | /user/{user_name}/avatar      | Posts an avatar, replacing the old one with multipart upload         | YES               |
| list            | GET    | /user/{user_name}/list        | Get the list for queried user                                        | YES               |
| put_list        | PUT    | /user/{user_name}/list        | Puts the send list from the `body` to the current user               | YES               |
