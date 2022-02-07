import React from 'react'
import { AxiosResponse } from 'axios'
import Cookies from 'js-cookie'

import { getApi } from '../apis/Requests'
import ResponseUser from "../intefaces/ResponseUser"

import LoginForm from "./LoginForm"
import LogoutForm from './LogoutForm'
import RegisterForm from "./RegisterForm"

import style from "./UserInfo.module.scss";


interface UserInfoState {
    responseUser: ResponseUser | null
    isLoggedIn: boolean
}


interface UserInfoProps {}


class UserInfo extends React.Component<UserInfoProps, UserInfoState> {

    constructor(props: UserInfoProps) {
        super(props)

        this.state = {
            responseUser: null,
            isLoggedIn: false,
        }
    }

    setIsLoggedIn(value: boolean): void {
        this.setState({isLoggedIn: value})
    }

    async apiGetUserInfo(): Promise<void> {
        const url = "/rest-auth/user"
        const result = (await getApi(url)) as AxiosResponse
        if (result.status === 200) {
            const responseUser: ResponseUser = {
                username: result.data.username,
                email: result.data.email,
                firstName: result.data.first_name,
                lastName: result.data.last_name,
            }
            this.setState({responseUser: responseUser, isLoggedIn: true})
        }
        else if (result.status === 401) {
            if (result.data.detail === "Invalid token.") {
                Cookies.remove("csrftoken")
            }
        }
        else {
            // TODO: add better error handling
            alert(`Failed to get user info. ${JSON.stringify(result.data)}`)
            
        }
    }

    async componentDidMount(): Promise<void> {
        await this.apiGetUserInfo()
    }

    render(): JSX.Element {
        if (this.state.isLoggedIn) {

            return <div className={style.userInfoBlock}>

                <LogoutForm setIsLoggedIn={this.setIsLoggedIn.bind(this)} />

                <div>Welcome, {this.state.responseUser?.username}</div>
    
            </div>

        }
        else {

            return <div className={style.userInfoBlock}>
                <LoginForm setIsLoggedIn={this.setIsLoggedIn.bind(this)} />
                <div className={style.verticalLine} />
                <RegisterForm />
            </div>
        }
    }

}


export default UserInfo