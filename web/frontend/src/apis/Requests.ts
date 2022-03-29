import axios, { AxiosRequestConfig, AxiosResponse } from "axios";
import Cookies from "js-cookie";

import { apiUrl } from "../config";

// TODO: use this if any request with auth token will be needed
// axios.defaults.headers.common['X-CSRFToken'] = Cookies.get('csrftoken')
// axios.defaults.headers.common['X-Requested-With'] = 'XMLHttpRequest'
const csrftoken = Cookies.get("csrftoken");
if (csrftoken) {
  axios.defaults.headers.common["Authorization"] = `Token ${csrftoken}`;
}

enum METHODS {
  "GET" = "GET",
  "POST" = "POST",
  "DELETE" = "DELETE",
}

const request = async (
  method: METHODS,
  path: string,
  data: any = null,
  noprefix = false,
  isFile = false
): Promise<AxiosResponse> => {
  let url = `${apiUrl}${path}`;

  if (noprefix) {
    url = path;
  }
  const req: Partial<AxiosRequestConfig> = { url, method };
  if (method === METHODS.GET && data) {
    req.params = data;
  } else if (data) {
    req.data = data;
  }
  if (isFile) {
    req.headers = {'Content-Type': 'multipart/form-data; boundary=----WebKitFormBoundaryXybQAkCayqX2b0uI'}
  }
  return axios(req);
};

export async function getApi(
  path: string,
  params: Array<string> | null = null,
  noprefix = false
): Promise<AxiosResponse | string> {
  const isFile = false
  try {
    return await request(METHODS.GET, path, params, noprefix, isFile);
  } catch (e: any) {
    return e.response as string;
  }
}

export async function postApi(
  path: string,
  data: any = null,
  noprefix = false,
  isFile = false
): Promise<AxiosResponse | string> {
  try {
    return await request(METHODS.POST, path, data, noprefix, isFile);
  } catch (e: any) {
    return e.response as string;
  }
}

export async function deletetApi(
  path: string,
  data: any = null
): Promise<AxiosResponse | string> {
  const noprefix = false
  const isFile = false
  try {
    return await request(METHODS.DELETE, path, data, noprefix, isFile);
  } catch (e: any) {
    return e.response as string;
  }
}
