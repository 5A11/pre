import React from "react";
import { AxiosResponse } from "axios";
import Cookies from "js-cookie";

import style from "./LoginForm.module.scss";

import { postApi } from "../apis/Requests";

interface LoginFormState {
    username: string;
    password: string;
}

interface LoginFormProps {
    setIsLoggedIn: (value: boolean) => void
}

class LoginForm extends React.Component<LoginFormProps, LoginFormState> {
    constructor(props: LoginFormProps) {
        super(props);

        this.state = {
            username: "",
            password: "",
        };
    }

    handleUsernameChange(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ username: event.target.value });
    }

    handlePasswordChange(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ password: event.target.value });
    }

    async handleSubmit(event: React.FormEvent<HTMLFormElement>): Promise<void> {
        event.preventDefault();
        await this.apiLogin();
    }

    async apiLogin(): Promise<void> {
        const url = "/rest-auth/login/";
        const data = {
            username: this.state.username,
            password: this.state.password,
        };
        const result = (await postApi(url, data)) as AxiosResponse;
        if (result.status === 200) {
            Cookies.set("csrftoken", `${result.data.key}`);
            this.props.setIsLoggedIn(true);
        } 
        else if (result.status === 401 && result.data.detail === "Invalid token.") {
            Cookies.remove("csrftoken")
        }
        else {
            // TODO: add better error handling
            alert(`Failed to login. ${JSON.stringify(result.data)}`);
        }
    }

    render(): JSX.Element {
        return (
            <form onSubmit={this.handleSubmit.bind(this)} className={style.Form}>
                <h1>Login</h1>
                <input
                    type="text"
                    placeholder="Username"
                    autoFocus
                    onChange={this.handleUsernameChange.bind(this)}
                />
                <input
                    type="password"
                    placeholder="Password"
                    onChange={this.handlePasswordChange.bind(this)}
                />
                <button
                    type="submit"
                    disabled={!(this.state.username && this.state.password)}
                >
                    Login
                </button>
            </form>
        );
    }
}

export default LoginForm;
