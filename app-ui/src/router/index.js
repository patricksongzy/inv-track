import { createRouter, createWebHistory } from 'vue-router';
import HomePage from '../views/HomePage.vue';
import ItemsPage from '../views/ItemsPage.vue';
import ItemPage from '../views/ItemPage.vue';
import LocationsPage from '../views/LocationsPage.vue';
import LocationPage from '../views/LocationPage.vue';
import TransactionsPage from '../views/TransactionsPage.vue';
import TransactionPage from '../views/TransactionPage.vue';

const routes = [
    { path: '/', component: HomePage },
    { path: '/items', component: ItemsPage },
    { path: '/items/:id', component: ItemPage },
    { path: '/locations', component: LocationsPage },
    { path: '/locations/:id', component: LocationPage },
    { path: '/transactions', component: TransactionsPage },
    { path: '/transactions/:id', component: TransactionPage },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;