import { Routes } from '@angular/router';
import { StartGuard } from '../shared/guards/start.guard';
import { AuthGuard } from '../shared/guards/auth.guard';
import { AnonGuard } from '../shared/guards/anon.guard';
import { LoginPageComponent } from '../ui/pages/login-page.component';
import { ProfilePageComponent } from '../ui/pages/profile-page.component';
import { ErrorPageComponent } from '../ui/pages/error-page.component';

export const APP_ROUTES: Routes = [
  { path: '', canActivate: [StartGuard], children: [] },
  { path: 'login', component: LoginPageComponent, canActivate: [AnonGuard] },
  { path: 'profile', component: ProfilePageComponent, canActivate: [AuthGuard] },
  { path: 'error', component: ErrorPageComponent },
  { path: '**', redirectTo: 'error' }
];
