import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { RouterLink, RouterOutlet } from '@angular/router';
import { AuthService } from '../shared/services/auth.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, RouterLink],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <header class="w-full border-b bg-white">
      <div class="max-w-4xl mx-auto p-4 flex items-center justify-between">
        <a routerLink="/" class="font-semibold">Rocy</a>
        <nav class="space-x-4">
          @if (!loggedIn()) {
            <a routerLink="/login" class="text-blue-600 hover:underline">Login</a>
          } @else {
            <a routerLink="/profile" class="text-blue-600 hover:underline">Profile</a>
          }
        </nav>
      </div>
    </header>
    <main class="max-w-4xl mx-auto p-4">
      <router-outlet></router-outlet>
    </main>
  `
})
export class AppComponent {
  private auth = inject(AuthService);
  readonly loggedIn = computed(() => !!this.auth.user());
}
