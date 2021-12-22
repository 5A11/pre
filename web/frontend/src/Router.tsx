import React from 'react'
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom'

import Home from './pages/Home'


const AppRouter = (): JSX.Element => (
  <Router>
    <Routes>
      <Route key="/" path="/" element={<Home />} />
    </Routes>
  </Router>
)

export default AppRouter
