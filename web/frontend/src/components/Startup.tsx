import React from 'react'
import { AxiosResponse } from 'axios'
import Cookies from 'js-cookie'

import { getApi } from '../apis/Requests'
import ResponseUser from "../intefaces/ResponseUser"

import Header from "./Header";
import LoginForm from "./LoginForm"
import RegisterForm from "./RegisterForm"
import DataAccessGranted from "./DataAccessGranted";
import DataAccessOwned from "./DataAccessOwned";

import style from "./Startup.module.scss";


interface StartupState {
    responseUser: ResponseUser | null
    isLoggedIn: boolean
}


interface StartupProps {}


class Startup extends React.Component<StartupProps, StartupState> {

    constructor(props: StartupProps) {
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

    renderBody():JSX.Element {
        if (this.state.isLoggedIn) {
            return <div className={style.Body}>
                <DataAccessOwned />
                <DataAccessGranted />
            </div>
        }
        else {
            return <div className={style.Body}>
                <LoginForm setIsLoggedIn={this.setIsLoggedIn.bind(this)} />
                <RegisterForm />
            </div>
        }
    }

    render(): JSX.Element {

        return <div className={style.userInfoBlock}>
            <Header 
                setIsLoggedIn={this.setIsLoggedIn.bind(this)}
                responseUser={this.state.responseUser}
                isLoggedIn={this.state.isLoggedIn}
            />
            {this.renderBody()}

        </div>
    }

}


export default Startup