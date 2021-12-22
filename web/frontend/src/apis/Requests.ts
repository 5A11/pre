import axios, { AxiosRequestConfig, AxiosResponse } from 'axios'
// import Cookies from 'js-cookie'

import {apiUrl} from '../config'

// TODO: use this if any request with auth token will be needed
// axios.defaults.headers.common['X-CSRFToken'] = Cookies.get('csrftoken')
// axios.defaults.headers.common['X-Requested-With'] = 'XMLHttpRequest'

enum METHODS {
  'GET' = 'GET',
  'POST' = 'POST'
}

const request = async (
  method: METHODS,
  path: string,
  data: any = null,
  noprefix = false
): Promise<AxiosResponse> => {
  let url = `${apiUrl}${path}`
  if (noprefix) {
    url = path
  }
  const req: Partial<AxiosRequestConfig> = { url, method }
  if (method === METHODS.GET && data) {
    req.params = data
  } else if (data) {
    req.data = data
  }
  return axios(req)
}

export async function getApi(
  path: string,
  params: Array<string> | null = null,
  noprefix = false
): Promise<AxiosResponse | string> {
  try {
    return await request(METHODS.GET, path, params, noprefix)
  } catch (e: any) {
    return e.response as string
  }
}

export async function postApi(
  path: string,
  data: any = null,
  noprefix = false
): Promise<AxiosResponse | string> {
  try {
    return await request(METHODS.POST, path, data, noprefix)
  } catch (e: any) {
    return e.response as string
  }
}
