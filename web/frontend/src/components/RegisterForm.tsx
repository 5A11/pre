import React from 'react'
import { AxiosResponse } from 'axios'

import { postApi } from '../apis/Requests'


interface RegisterFormState {
    username: string
    email: string
    password1: string
    password2: string
}


interface RegisterFormProps { }


class RegisterForm extends React.Component<RegisterFormProps, RegisterFormState> {

    constructor(props: RegisterFormProps) {
        super(props)

        this.state = {
            username: "",
            email: "",
            password1: "",
            password2: "",
        }
    }

    handleUsernameChange(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ username: event.target.value })
    }

    handleEmailChange(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ email: event.target.value })
    }

    handlePassword1Change(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ password1: event.target.value })
    }

    handlePassword2Change(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ password2: event.target.value })
    }

    isFormDisabled(): boolean {
        return !(
            this.state.username
            && this.state.email
            && this.state.password1
            && this.state.password2
        )
    }

    async handleSubmit(event: React.FormEvent<HTMLFormElement>): Promise<void> {
        event.preventDefault()
        await this.apiRegister()
    }

    async apiRegister(): Promise<void> {
        const url = "/rest-auth/registration/"
        const data = {
            "username": this.state.username,
            "email": this.state.email,
            "password1": this.state.password1,
            "password2": this.state.password2,
        }
        const result = (await postApi(url, data)) as AxiosResponse
        if (result.status === 200) {
            alert("Registration successful!")
        }
        else {
            // TODO: add better error handling
            alert(`Failed to login. ${JSON.stringify(result.data)}`)
        }
    }

    render(): JSX.Element {
        return <form
            onSubmit={this.handleSubmit.bind(this)}
        >
            <h1>Register</h1>
            <input
                type="text"
                placeholder="Username"
                autoFocus
                onChange={this.handleUsernameChange.bind(this)}
            />
            <input
                type="text"
                placeholder="Email"
                autoFocus
                onChange={this.handleEmailChange.bind(this)}
            />
            <input
                type="password"
                placeholder="Password"
                onChange={this.handlePassword1Change.bind(this)}
            />
            <input
                type="password"
                placeholder="Confirm password"
                onChange={this.handlePassword2Change.bind(this)}
            />
            <button
                type="submit"
                disabled={this.isFormDisabled()}
            >
                Submit
            </button>
        </form>
    }

}


export default RegisterForm