import { JsonPipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, OnInit } from '@angular/core';
import { AuthService } from '../../shared/services/auth.service';
import { UploadImageComponent } from '../components/upload-image/upload-image.component';
import { ImageListComponent } from '../components/image-list/image-list.component';

@Component({
    selector: 'app-profile-page',
    standalone: true,
    imports: [JsonPipe, UploadImageComponent, ImageListComponent],
    changeDetection: ChangeDetectionStrategy.OnPush,
    template: `
        <section class="mt-6">
            <h1 class="text-2xl font-bold mb-4">Your Profile</h1>
            @if (auth.loading()) {
                <div class="text-gray-500">Loading...</div>
            } @else {
                @if (auth.user()) {
                    <div class="space-y-4">
                        <div class="flex items-center space-x-4">
                            <div>
                                <div class="font-semibold">{{ auth.user()?.name || auth.user()?.email || 'User' }}</div>
                                <div class="text-sm text-gray-600">{{ auth.user()?.email }}</div>
                            </div>
                        </div>
                        <pre class="bg-gray-100 p-3 rounded border text-sm overflow-auto">{{ auth.user() | json }}</pre>
                        <app-upload-image (uploaded)="onUploaded($event); imageList.refresh()"></app-upload-image>
                        <app-image-list #imageList></app-image-list>
                        <button (click)="logout()" class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700">Logout</button>
                    </div>
                } @else {
                    <div class="text-gray-700">No user data. You may need to log in again.</div>
                }
            }
        </section>
    `
})
export class ProfilePageComponent implements OnInit {
    auth = inject(AuthService);

    ngOnInit(): void {
        if (!this.auth.user()) {
            this.auth.me().subscribe();
        }
    }

    onUploaded(_res: unknown) {
        // Refresh the profile after upload to reflect any new picture or data
        this.auth.me().subscribe();
    }

    logout() {
        this.auth.logout();
    }
}
